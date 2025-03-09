use std::{collections::HashMap, sync::Arc};

use db::Ctx;
use search::words::{best_ngram_match, filter_stop_words, finalize_word_list};
use tokio::task;

pub mod search;
pub mod http_net;
pub mod url;
pub mod crawler;
pub mod tfidf;
pub mod html;
pub mod db;

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
    println!("SERVING...");

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
            let similar_keys: Vec<String> = Ctx::get_partially_matching_keys(&db, search_query);
            let best_match: Option<String> = best_ngram_match(search_query, &similar_keys);

            if let Some(a_match) = best_match {
                if let Some(db_match) = db.get(a_match)? {
                    let response = String::from_utf8_lossy(&db_match);
                    let mut response = tiny_http::Response::from_string(response);
                    for header in &cors_headers {
                        response.add_header(header.clone());
                    }

                    request.respond(response).unwrap();
                }
            }
            else {
                let error_response = "Not Found";
                let mut response = tiny_http::Response::from_string(error_response);
                for header in &cors_headers {
                    response.add_header(header.clone());
                }
                request.respond(response).unwrap();
            }
        }
    }
    Ok(())
}

async fn index_search(db: Arc<rocksdb::DB>) -> Result<(), Box<dyn std::error::Error>> {
    let seend_index: &str = "https://www.tutorialspoint.com/python/index.htm";

    // set up custom headers (User-Agent is important for avoiding bot detection)
    let mut reqw_headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    reqw_headers.insert(
        reqwest::header::USER_AGENT, 
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3"
        )
    );

    let req_client: reqwest::Client = reqwest::Client::new();
    let resp: reqwest::Response = req_client.get(seend_index).headers(reqw_headers).send().await?;
    
    if !resp.status().is_success() {
        panic!("Failed to make a request!");
    }

    let mut tasks: Vec<task::JoinHandle<()>> = vec![];

    let document: scraper::Html = scraper::Html::parse_document(&resp.text().await?);
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

fn create_page_index(db: &rocksdb::DB, page: &html::HtmlDoc) -> Result<i32, rocksdb::Error> {
    let splited_title: Vec<&str> = page.title.split(&['-', ' ', ':', '@'][..]).collect::<Vec<&str>>();
    let non_stop_words_list: Vec<&str> = filter_stop_words(splited_title);
    let processed_words: Vec<String> = finalize_word_list(search::stemming::stem(non_stop_words_list));
    // println!("{:?}", processed_words);
    // return Ok(0);

    let new_entry = serde_json::json!({
        "url": format!("{}", page.url),
        "title": format!("{}", page.title)
    });
    println!("{:?}", new_entry);

    for word in processed_words {
        match db.get(&word) {
            Ok(Some(existing_value)) => {
                let mut existing_data: Vec<serde_json::Value> = serde_json::from_slice(&existing_value).unwrap_or_else(|_| vec![]);
                
                if !existing_data.iter().any(|e| e["url"] == page.url) {
                    println!("Data already exists: {:?}", existing_data);
                    existing_data.push(new_entry.clone());
                    println!("modifying already exists: {:?}", existing_data);
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

async fn fetch_page_title(url: &str) -> Result<html::HtmlDoc, reqwest::Error> {
    let resp: String = reqwest::get(url).await?.text().await?;
    let document: scraper::Html = scraper::Html::parse_document(&resp);

    let title_selector: scraper::Selector = scraper::Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|t| t.inner_html())
        .unwrap_or_else(|| "No Title".to_string());

    Ok(html::HtmlDoc {
        title,
        url: url.to_string(),
    })
}

async fn start_engine(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
// Setting up a database.
    // Database is used for indexing.
    let db_path: &str = "spy-db";
    
    let mut opts = rocksdb::Options::default();
    opts.create_if_missing(true);

    let db = Arc::new(rocksdb::DB::open_default(db_path).expect("Failed!!!"));

    if let Some(action) = args.get(1) {
        if action == "index" {
            println!("Started indexing...");
            index_search(db).await?;
        } else {
            server_search(db)?;
        }
    } else {
        server_search(db)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    start_engine(&args).await

    // let words = ["python", "best", "language", "world", "python", "programming", "best"];
    // println!("{:?}", compute_tfidf_score(&words));

    // Ok(())
}