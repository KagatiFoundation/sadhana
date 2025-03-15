use std::{
    collections::{HashMap, HashSet}, hash::Hash, sync::Arc
};

use redis::Commands;
use search::words::{
    best_ngram_match, 
    filter_stop_words, 
    finalize_word_list
};
use tfidf::compute_tfidf_score;
use tokio::task;
use ::url::Url;

pub mod search;
pub mod http_net;
pub mod url;
pub mod crawler;
pub mod tfidf;
pub mod html;
pub mod ctx;

pub const REDIS_TERMS_KEY: &str = "term_frequencies";

fn parse_query_params(url: &str) -> HashMap<String, String> {
    let mut params: HashMap<String, String> = HashMap::new();
    
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
            let similar_keys: Vec<String> = ctx::Ctx::get_partially_matching_keys(&db, search_query);
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

async fn index_search(ctxx: &Arc<ctx::Ctx>) -> Result<(), Box<dyn std::error::Error>> {
    // let seed_main_domains = ["https://docs.python.org/3", "https://docs.oracle.com/javase/tutorial"];
    let seed_index: &str = "https://docs.python.org/3";

    // set up custom headers (User-Agent is important for avoiding bot detection)
    let mut reqw_headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    reqw_headers.insert(
        reqwest::header::USER_AGENT, 
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3"
        )
    );

    let req_client: reqwest::Client = reqwest::Client::new();
    let resp: reqwest::Response = req_client.get(seed_index).headers(reqw_headers).send().await?;
    
    if !resp.status().is_success() {
        panic!("Failed to make a request!");
    }

    let mut tasks: Vec<task::JoinHandle<()>> = vec![];

    let document: scraper::Html = scraper::Html::parse_document(&resp.text().await?);
    let selector: scraper::Selector = scraper::Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        if let Some(href_content) = element.attr("href") {
            let url_clone: String = String::from(href_content);
            let ctx_clone = Arc::clone(ctxx);

            if url::is_valid_url(href_content) {
                let page_fetch_task: task::JoinHandle<()> = task::spawn(async move {
                    if let Ok(Some(page)) = fetch_page_title(&url_clone).await {
                        let _ = create_page_index(&ctx_clone, &page);
                    }
                });
                tasks.push(page_fetch_task);
            }
            else {
                let base_url: Url = Url::parse(seed_index).expect("Invalid base URL");
                let abs_url: Url = base_url.join(href_content).expect("Invalid URL formation!");
                let page_fetch_task: task::JoinHandle<()> = task::spawn(async move {
                    if let Ok(Some(ref mut page)) = fetch_page_title(abs_url.as_str()).await {
                        page.proprocess();
                        let _ = create_page_index(&ctx_clone, page);
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

fn analyze_keyword_relevance<'a>(
    title_tfidf: &HashMap<&'a str, f32>, 
    content_tf_idf: &HashMap<&'a str, f32>,
    title_weight: f32,
    content_weight: f32
) -> Vec<(&'a str, f32)> {
    let mut all_tfidf_keys: HashSet<&str> = title_tfidf.keys().cloned().collect();
    all_tfidf_keys.extend(content_tf_idf.keys().cloned());

    let mut combined_tfidf: HashMap<&str, f32> = HashMap::new();

    for key in all_tfidf_keys {
        let title_score = *title_tfidf.get(key).unwrap_or(&0.0);
        let content_score = *content_tf_idf.get(key).unwrap_or(&0.0);

        let final_score: f32 = (title_score * title_weight) + (content_score * content_weight);
        combined_tfidf.entry(key).or_insert(final_score);
    }

    let mut sorted_terms: Vec<(&str, f32)> = combined_tfidf.into_iter().collect();
    sorted_terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted_terms
}

fn create_page_index(ctxx: &ctx::Ctx, page: &html::HtmlDoc) -> Result<i32, rocksdb::Error> {
    if page.title == "No Title" {
        return Ok(-1);
    }

    let splited_title: Vec<&str> = page.title.split(&['-', ' ', ':', '@'][..]).collect::<Vec<&str>>();

    let non_stop_words_list: Vec<&str> = filter_stop_words(splited_title);
    let processed_words: Vec<String>= finalize_word_list(search::stemming::stem(non_stop_words_list));

    let splited_content: Vec<&str> = page.content.split(&[' ', '\n', '\t'][..]).collect::<Vec<&str>>();
    let content_words: Vec<&str> = filter_stop_words(splited_content);
    let processed_content_words: Vec<String>= finalize_word_list(search::stemming::stem(content_words));

    let tfidf_scores: HashMap<&str, f32> = compute_tfidf_score(ctxx, &processed_words.iter().map(AsRef::as_ref).collect());
    let content_tfidf: HashMap<&str, f32> = compute_tfidf_score(ctxx, &processed_content_words.iter().map(AsRef::as_ref).collect());
    
    //println!("{:?}\n\n{:?}\n\n", tfidf_scores, content_tfidf);

    let final_keywords = 
        analyze_keyword_relevance(
            &tfidf_scores, 
            &content_tfidf, 
            0.2, 
            0.8
        );

    for (word, tf_idf) in final_keywords.iter().take(3) {
        let new_entry = serde_json::json!({
            "url": format!("{}", page.url),
            "title": format!("{}", page.title),
            "score": *tf_idf
        });

        match ctxx.rocks_con.get(word) {
            Ok(Some(existing_value)) => {
                let mut existing_data: Vec<serde_json::Value> = serde_json::from_slice(&existing_value).unwrap_or_else(|_| vec![]);
                println!("KEY: {}\nVALUE: {:?}\n\n", word, existing_data);
                let mut incr_doc_count: bool = false;
                
                if !existing_data.iter().any(|e| e["url"] == page.url) {
                    existing_data.push(new_entry.clone());

                    _ = store_term_freq(ctxx, word);

                    let updated_value: Vec<u8> = serde_json::to_vec(&existing_data).unwrap();
                    ctxx.rocks_con.put(word, updated_value)?;

                    incr_doc_count = true;
                }

                if incr_doc_count {
                    _ = ctxx.incr_doc_count();
                }
            }
            _ => {
                let new_data: Vec<u8> = serde_json::to_vec(&vec![new_entry.clone()]).unwrap();
                ctxx.rocks_con.put(word, new_data)?;
                _ = store_term_freq(ctxx, word);
                _ = ctxx.incr_doc_count();
            }
        }
    }
    Ok(0)
}

fn store_term_freq(ctxx: &ctx::Ctx, term: &str) -> redis::RedisResult<()> {
    let conn = &mut ctxx.redis_con.get_connection().ok().unwrap();
    conn.hincr(REDIS_TERMS_KEY, term, 1)
}

async fn fetch_page_title(url: &str) -> Result<Option<html::HtmlDoc>, reqwest::Error> {
    let http_resp: reqwest::Response = reqwest::get(url).await?;

    if http_resp.status() == 404 || http_resp.status() == 301 {
        return Ok(None);
    }

    let resp: String = http_resp.text().await?;
    let document: scraper::Html = scraper::Html::parse_document(&resp);

    let title_selector: scraper::Selector = scraper::Selector::parse("title").unwrap();
    let title: String = document
        .select(&title_selector)
        .next()
        .map(|t| t.inner_html())
        .unwrap_or_else(|| "No Title".to_string());

    let p_tags_selector: scraper::Selector = scraper::Selector::parse("p").unwrap();

    let mut content: String = String::from("");

    for p_tag in document.select(&p_tags_selector) {
        let p_tag_content: Vec<String> = p_tag
                                                .text()
                                                .collect::<Vec<&str>>()
                                                .iter()
                                                .map(|content| String::from(*content))
                                                .collect::<Vec<String>>();
        content.push_str(p_tag_content.join(" ").as_str());
    }

    Ok(Some(html::HtmlDoc::with_content(title, url.to_string(), content)))
}

async fn start_engine(ctxx: Arc<ctx::Ctx>, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(action) = args.get(1) {
        if action == "index" {
            println!("Started indexing...");
            index_search(&ctxx).await?;
        } else {
            server_search(Arc::clone(&ctxx.rocks_con))?;
        }
    } else {
        server_search(Arc::clone(&ctxx.rocks_con))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let context: Arc<ctx::Ctx> = Arc::new(ctx::Ctx::new(ctx::CtxOptions::default()));

    start_engine(context, &args).await

    // let words = ["python", "best", "language", "world", "python", "programming", "best"];
    // println!("{:?}", compute_tfidf_score(&words));

    // Ok(())
}