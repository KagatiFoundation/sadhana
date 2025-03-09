use std::collections::HashSet;

fn generate_ngrams(word: &str, sz: usize) -> HashSet<String> {
    let mut ngrams: HashSet<String> = HashSet::new();
    let chars: Vec<char> = word.chars().collect();

    if chars.len() < sz {
        ngrams.insert(word.to_string());
        return ngrams;
    }

    for i in 0..=(chars.len() - sz) {
        ngrams.insert(chars[i..i + sz].iter().collect());
    }
    ngrams
}

pub fn best_ngram_match(word: &str, word_list: &[String]) -> Option<String> {
    let query_ngram: HashSet<String> = generate_ngrams(word, 3);
    let mut best_match: Option<String> = None;
    let mut max_overlap: usize = 0;

    for key in word_list {
        let key_ngram: HashSet<String> = generate_ngrams(key, 3);
        let overlap: usize = query_ngram.intersection(&key_ngram).count();
        if overlap > max_overlap {
            max_overlap = overlap;
            best_match = Some(key.clone());
        }
    }
    best_match
}