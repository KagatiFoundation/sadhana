use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref STOP_WORDS: HashSet<&'static str> = {
        HashSet::from([
            "a", "about", "above", "across", "after", "afterwards", "again", "against",
            "all", "almost", "alone", "along", "already", "also", "although", "always",
            "am", "among", "amongst", "amount", "an", "and", "another", "any", "anyhow",
            "anyone", "anything", "anyway", "anywhere", "are", "around", "as", "at",
            "back", "be", "became", "because", "become", "becomes", "becoming", "been",
            "before", "beforehand", "behind", "being", "below", "beside", "besides",
            "between", "beyond", "both", "bottom", "by", "call", "can", "cannot",
            "ca", "could", "did", "do", "does", "doing", "done", "down", "due", "during",
            "each", "eight", "either", "eleven", "else", "elsewhere", "empty", "enough",
            "even", "ever", "every", "everyone", "everything", "everywhere", "except",
            "few", "fifteen", "fifty", "fill", "find", "fire", "first", "five", "for",
            "former", "formerly", "forty", "found", "four", "from", "front", "full",
            "further", "get", "give", "go", "got", "had", "has", "have", "he", "hence",
            "her", "here", "hereafter", "hereby", "herein", "hereupon", "hers", "herself",
            "him", "himself", "his", "how", "however", "hundred", "i", "if", "in", "inc",
            "indeed", "into", "is", "it", "its", "itself", "just", "keep", "last", "latter",
            "latterly", "least", "less", "like", "likely", "limited", "little", "ll",
            "look", "ltd", "made", "many", "may", "me", "meanwhile", "might", "mill",
            "mine", "more", "moreover", "most", "mostly", "move", "much", "must", "my",
            "myself", "name", "namely", "neither", "never", "nevertheless", "new",
            "next", "nine", "no", "nobody", "none", "noone", "nor", "not", "nothing",
            "now", "nowhere", "of", "off", "often", "on", "once", "one", "only", "onto",
            "or", "other", "others", "otherwise", "our", "ours", "ourselves", "out",
            "over", "own", "part", "particular", "per", "perhaps", "please", "put",
            "rather", "re", "really", "regarding", "same", "say", "see", "seem", "seemed",
            "seeming", "seems", "serious", "several", "she", "should", "show", "side",
            "since", "six", "sixty", "so", "some", "somehow", "someone", "something",
            "sometime", "sometimes", "somewhere", "still", "such", "take", "ten", "than",
            "that", "the", "their", "them", "themselves", "then", "thence", "there",
            "thereafter", "thereby", "therefore", "therein", "thereupon", "these",
            "they", "thick", "thin", "third", "this", "those", "though", "three",
            "through", "throughout", "thru", "thus", "to", "together", "too", "top",
            "toward", "towards", "twelve", "twenty", "two", "un", "under", "until",
            "up", "upon", "us", "very", "via", "was", "we", "well", "were", "what",
            "whatever", "when", "whence", "whenever", "where", "whereafter", "whereas",
            "whereby", "wherein", "whereupon", "wherever", "whether", "which", "while",
            "whither", "who", "whoever", "whole", "whom", "whose", "why", "will",
            "with", "within", "without", "would", "yet", "you", "your", "yours",
            "yourself", "yourselves", "&amp;"
        ])
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
    let re = regex::Regex::new(r"[^a-zA-Z]").unwrap();
    let no_spec: String = re.replace_all(word, " ").trim().to_string();
    if no_spec.contains(" ") {
        no_spec.split_whitespace().collect::<Vec<&str>>().first().unwrap().to_string()
    } else {
        no_spec
    }
}

pub fn finalize_word_list(word_list: Vec<&str>) -> Vec<String> {
    word_list
        .iter()
        .map(|word| remove_special_characters(word.to_lowercase().as_str()))
        .filter(|word| !word.is_empty())
        .collect::<Vec<String>>()
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