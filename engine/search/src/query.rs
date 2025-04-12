use context::CTX;

use rocksdb::{self};
use wp::best_ngram_match;

pub fn query_rdb(ctx: &CTX, query: &str) -> Result<String, rocksdb::Error> {
    if let Some(value) = ctx.rocks_con.get(query)? {
        let response_str = String::from_utf8_lossy(&value);
        return Ok(response_str.to_string());
    }
    else {
        let similar_keys: Vec<String> = CTX::get_partially_matching_keys(&ctx.rocks_con, query);
        let best_match: Option<String> = best_ngram_match(query, &similar_keys);

        if let Some(a_match) = best_match {
            if let Some(db_match) = ctx.rocks_con.get(a_match)? {
                let response = String::from_utf8_lossy(&db_match);
                return Ok(response.to_string());
            }
        }
        else {
            return Ok(String::new());
        }
    }
    Ok(String::new())
} 