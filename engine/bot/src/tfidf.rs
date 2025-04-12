use std::collections::HashMap;

use context::*;
use redis::Commands;

fn tf_score(word_count: usize, total_count: usize) -> f32 {
    word_count as f32 / total_count as f32
}

// param word_count: number of documents the word appears in
fn idf_score(total_docs: usize, word_count: usize) -> f32 {
    (total_docs as f32 / word_count as f32).log10()
}

fn word_frequency(word: &str, word_list: &Vec<&str>) -> usize {
    let mut count: usize = 0;
    for w in word_list {
        if *w == word {
            count += 1;
        }
    }
    count
}

pub fn compute_tfidf_score<'a>(ctxx: &CTX, word_list: &Vec<&'a str>) -> HashMap<&'a str, f32> {
    let mut tf_scores: HashMap<&str, f32> = HashMap::new();
    if word_list.is_empty() {
        return tf_scores;
    }

    let mut redis_con = ctxx.redis_con.get_connection().ok().unwrap();
    let total_docs: usize = ctxx.get_internal_value::<usize>("total_docs").unwrap_or(1);
    
    let total_count: usize = word_list.len();
    for word in word_list {
        // frequency of word in the current list
        let curr_word_freq: usize = word_frequency(word, word_list);

        // try to get the term's appearances
        let term_appearance: Result<usize, redis::RedisError> = redis_con.hget(REDIS_TERMS_KEY, word);

        // how many documents does the term appears in
        let total_term_freq: usize = term_appearance.ok().unwrap_or(1);

        tf_scores.insert(
            word, 
            tf_score(curr_word_freq, total_count) * idf_score(total_docs, total_term_freq)
        );
    }
    tf_scores
}