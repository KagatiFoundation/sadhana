pub fn stem(word_list: Vec<&str>) -> Vec<&str> {
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