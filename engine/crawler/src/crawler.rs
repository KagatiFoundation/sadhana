use std::{
    collections::{
        HashSet,
        VecDeque
    }, 
    error::Error, 
    sync::{
        Arc, 
        Mutex
    }
};

use indexer::Indexer;

use crate::HtmlDoc;

#[derive(Debug, Clone)]
pub struct CrawlerOptions {
    pub max_depth: usize,

    pub seed_url: String,
}

#[derive(Debug, Clone)]
pub struct Crawler {
    pub index: Arc<Indexer>,
    
    /// Options
    options: CrawlerOptions,

    links_to_crawl: Arc<Mutex<VecDeque<String>>>,

    visited: Arc<Mutex<HashSet<String>>>
}

impl Crawler {
    pub fn new(indexer: Arc<Indexer>, opts: CrawlerOptions) -> Self {
        let mut initial_links = VecDeque::new();
        initial_links.push_back(opts.seed_url.clone());

        Self {
            options: opts,
            index: indexer,
            links_to_crawl: Arc::new(Mutex::new(initial_links)),
            visited: Arc::new(Mutex::new(HashSet::new()))
        }
    }

    pub async fn start_crawling(&self) -> Result<(), Box<dyn Error>> {
        let mut depth: usize = 0;

        while depth <= self.options.max_depth {
            let mut handles = vec![];
            let mut batch = vec![];

            {
                let mut queue = self.links_to_crawl.lock().unwrap();
                let mut seen = self.visited.lock().unwrap();

                while let Some(link) = queue.pop_front() {
                    if !seen.contains(&link) {
                        seen.insert(link.clone());
                        batch.push(link.clone());
                    }
                }
            }

            for link in batch {
                if let Ok(handle) = Self::crawl_link(link, self.options.seed_url.clone(), self.links_to_crawl.clone()).await {
                    handles.push(handle);
                }
            }

            for handle in handles {
                let _ = handle.await;
            }
            depth += 1;
        }

        Ok(())
    }

    pub async fn crawl_link(
        link: String, 
        seed_url: String, 
        links_to_crawl: Arc<Mutex<VecDeque<String>>>
    ) -> Result<tokio::task::JoinHandle<()>, Box<dyn Error>> {
        let handle = tokio::task::spawn(async move {
            let links = if let Ok(page) = Self::fetch_html(link).await {
                page.extract_links()
            }
            else {
                return;
            };

            let mut queue = links_to_crawl.lock().unwrap();

            for href in links {
                let href_parsed = if url::Url::parse(&href).is_ok() {
                    href
                }
                else {
                    let base_url: url::Url = url::Url::parse(&seed_url).expect("Invalid base URL");
                    let abs_url: url::Url = base_url.join(&href).expect("Invalid URL formation!");
                    abs_url.as_str().to_string()
                };
    
                queue.push_back(href_parsed);
            }
        });
        Ok(handle)
    }

    /// Creates a HtmlDoc given a link.
    async fn fetch_html(link: String) -> Result<HtmlDoc, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .build()
            .expect("Failed to create client");

        let resp = client.get(link.clone()).send().await?;

        if !resp.status().is_success() {
            return Err(format!("Failed to fetch: {}", &link).into());
        }

        let page_body: String = resp.text().await?;
        Ok(HtmlDoc::parse(link, page_body))
    }

    fn create_index(_page: HtmlDoc) {

    }

    pub fn max_depth(&self) -> usize {
        self.options.max_depth
    }
}