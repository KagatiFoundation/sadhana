use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref STOP_WORDS: HashSet<&'static str> = {
        let mut stop_words = HashSet::new();
        let words = [
            "a", "an", "the", "in", "is", "was", "are", "were", "be", "been", "being", 
            "am", "have", "has", "had", "do", "does", "did", "to", "of", "and", "but", "or",
            "as", "at", "for", "by", "on", "with", "about", "against", "between", "into",
            "through", "during", "before", "after", "above", "below", "under", "over", "again",
            "further", "then", "once", "here", "there", "when", "where", "why", "how",
            "&amp;"
        ];

        for word in words.iter() {
            stop_words.insert(*word);
        }
        stop_words
    };

    static ref UNWANTED_SPECIAL_CHARS: HashSet<&'static str> = {
        let mut special_chars = HashSet::new();
        let words = [
            ".", "-", "_", "·", "—", "(", ")", "{", "}", "?", "<", ">"
        ];

        for word in words.iter() {
            special_chars.insert(*word);
        }
        special_chars
    };
}

fn remove_special_characters(word: &str) -> String {
    let re = regex::Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(word, "").to_string()
}

pub fn finalize_word_list(word_list: Vec<&str>) -> Vec<String> {
    word_list.iter().map(|word| -> String {
        remove_special_characters(word.to_lowercase().as_str())
    }).collect::<Vec<String>>()
}

pub fn filter_stop_words(word_list: Vec<&str>) -> Vec<&str> {
    word_list.into_iter()
        .filter(|&word| -> bool {
            let lower_word: String = word.to_lowercase();
            if UNWANTED_SPECIAL_CHARS.contains(lower_word.as_str()) {
                return false;
            }
            !word.is_empty() && 
            !STOP_WORDS.contains(lower_word.as_str())
        }
    ).collect()
}