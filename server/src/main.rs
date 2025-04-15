use std::sync::Arc;

use crawler::{Crawler, CrawlerOptions};
use indexer::Indexer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let indexer: Arc<Indexer> = Arc::new(Indexer{});

    let cr: Crawler = Crawler::new(
        indexer.clone(), 
        CrawlerOptions { 
            max_depth: 1, 
            seed_url: "https://docs.python.org/3/reference/compound_stmts.html".to_string(), 
        }
    );

    cr.start_crawling().await
}