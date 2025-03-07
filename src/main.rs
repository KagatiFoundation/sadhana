use std::{collections::HashMap, sync::Arc};

use tokio::{task, time::error};

pub mod search;
pub mod http_net;
pub mod url;

#[derive(Debug)]
struct HtmlPage {
    pub title: String,
    pub url: String
}

fn parse_query_params(url: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if let Some(query_string) = url.split('?').nth(1) {
        for pair in query_string.split('&') {
            let mut key_value = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
    }
    params
}

fn server_search(db: Arc<rocksdb::DB>) -> Result<(), rocksdb::Error> {
    let server: tiny_http::Server = http_net::start_local_server(5000);
    for request in server.incoming_requests() {
        println!("NEW REQUEST");

        let cors_headers: Vec<tiny_http::Header> = vec![
            tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(), 
            tiny_http::Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS").unwrap(),
            tiny_http::Header::from_bytes("Access-Control-Allow-Headers", "Content-Type, Authorization").unwrap(),
        ];

        let params: HashMap<String, String> = parse_query_params(request.url());
        let search_query: &String = if params.contains_key("query") {
            params.get("query").unwrap()
        } else {
            &String::from("wikipedia")
        };

        if *request.method() == tiny_http::Method::Options {
            let mut response = tiny_http::Response::empty(200);
            for header in &cors_headers {
                response.add_header(header.clone());
            }
            request.respond(response).unwrap();
            println!("Responded to OPTIONS");
        } else if let Some(value) = db.get(search_query)? {
            let response_str = String::from_utf8_lossy(&value);

            let mut response = tiny_http::Response::from_string(response_str);
            for header in &cors_headers {
                response.add_header(header.clone());
            }

            request.respond(response).unwrap();
        } else {
            let error_response = "Not Found";
            let mut response = tiny_http::Response::from_string(error_response);
            for header in &cors_headers {
                response.add_header(header.clone());
            }
            request.respond(response).unwrap();
        }
    }

    println!("SERVING...");
    Ok(())
}

async fn index_search(db: Arc<rocksdb::DB>) -> Result<(), Box<dyn std::error::Error>> {
    let resp: String = reqwest::get("https://stackoverflow.com/questions/tagged/python?tab=Votes")
        .await?
        .text()
        .await?;
    
    let mut tasks: Vec<task::JoinHandle<()>> = vec![];

    let document: scraper::Html = scraper::Html::parse_document(&resp);
    let selector: scraper::Selector = scraper::Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        if let Some(href_content) = element.attr("href") {
            let url_clone: String = String::from(href_content);
            let db_clone = Arc::clone(&db);

            if url::is_valid_url(href_content) {
                let page_fetch_task: task::JoinHandle<()> = task::spawn(async move {
                    if let Ok(page) = fetch_page_title(&url_clone).await {
                        let _ = create_page_index(&db_clone, &page);
                    }
                });
                tasks.push(page_fetch_task);
            }
        }
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("INDEXING DONE!!!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    return Ok(());

    // Setting up a database.
    // Database is used for indexing.
    let db_path: &str = "spy-db";
    
    let mut opts = rocksdb::Options::default();
    opts.create_if_missing(true);

    let db = Arc::new(rocksdb::DB::open_default(db_path).expect("Failed!!!"));
    // index_search(db).await?;
    server_search(db)?;
    Ok(())
}

fn create_page_index(db: &rocksdb::DB, page: &HtmlPage) -> Result<i32, rocksdb::Error> {
    let splited_title: Vec<&str> = page.title.split(" ").collect::<Vec<&str>>();
    let non_stop_words_list: Vec<&str> = remove_stop_words(splited_title);
    let processed_words: Vec<String> = lowercase_word_list(stem_words(non_stop_words_list));

    let new_entry = serde_json::json!({
        "url": format!("{}", page.url),
        "title": format!("{}", page.title)
    });

    for word in processed_words {
        match db.get(&word) {
            Ok(Some(existing_value)) => {
                let mut existing_data: Vec<serde_json::Value> = serde_json::from_slice(&existing_value).unwrap_or_else(|_| vec![]);
                
                if !existing_data.iter().any(|e| e["url"] == page.url) {
                    existing_data.push(new_entry.clone());
                }

                let updated_value = serde_json::to_vec(&existing_data).unwrap();
                db.put(&word, updated_value)?;
            }
            _ => {
                let new_data = serde_json::to_vec(&vec![new_entry.clone()]).unwrap();
                db.put(&word, new_data)?;
            }
        }
    }
    Ok(0)
}

fn lowercase_word_list(word_list: Vec<&str>) -> Vec<String> {
    word_list.iter().map(|word| word.to_lowercase()).collect::<Vec<String>>()
}

fn stem_words(word_list: Vec<&str>) -> Vec<&str> {
    let mut result_words: Vec<&str> = vec![];

    for word in word_list {
        let stemmed_word = if word.ends_with("ing") {
            word.strip_suffix("ing").unwrap()
        } else if word.ends_with("ed") {
            word.strip_suffix("ed").unwrap()
        } else if word.ends_with("ly") {
            word.strip_suffix("ly").unwrap()
        } else {
            word
        };
        result_words.push(stemmed_word);
    }
    result_words
}


fn remove_stop_words(word_list: Vec<&str>) -> Vec<&str> {
    let mut result_words: Vec<&str> = vec![];
    for word in word_list {
        if is_a_stop_word(word) {
            continue;
        }
        result_words.push(word);
    }
    result_words
}

fn is_a_stop_word(word: &str) -> bool {
    is_word_a_linking_verb(word) || is_word_an_article(word)
}

fn is_word_a_linking_verb(word: &str) -> bool {
    word == "is" || word == "was"
}

fn is_word_an_article(word: &str) -> bool {
    word == "a" || word == "the" || word == "an"
}

async fn fetch_page_title(url: &str) -> Result<HtmlPage, reqwest::Error> {
    let resp: String = reqwest::get(url).await?.text().await?;
    let document: scraper::Html = scraper::Html::parse_document(&resp);

    let title_selector: scraper::Selector = scraper::Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|t| t.inner_html())
        .unwrap_or_else(|| "No Title".to_string());

    Ok(HtmlPage {
        title,
        url: url.to_string(),
    })
}