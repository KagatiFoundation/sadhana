use std::collections::HashMap;

const DOCS_COUNT: usize = 100;

fn tf_score(word_count: usize, total_count: usize) -> f32 {
    word_count as f32 / total_count as f32
}

// param word_count: number of documents the word appears in
fn idf_score(word_count: usize) -> f32 {
    (DOCS_COUNT as f32 / word_count as f32).log10()
}

fn word_frequency(word: &str, word_list: &[&str]) -> usize {
    let mut count: usize = 0;
    for w in word_list {
        if *w == word {
            count += 1;
        }
    }
    count
}

pub fn compute_tfidf_score<'a>(word_list: &[&'a str]) -> HashMap<&'a str, f32> {
    let mut tf_scores: HashMap<&str, f32> = HashMap::new();
    if word_list.is_empty() {
        return tf_scores;
    }
    let total_count: usize = word_list.len();
    for word in word_list {
        let word_freq: usize = word_frequency(word, word_list);
        tf_scores.insert(
            word, 
            tf_score(word_freq, total_count) * idf_score(word_freq)
        );
    }
    tf_scores
}