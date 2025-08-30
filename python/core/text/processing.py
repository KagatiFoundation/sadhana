import spacy

nlp = spacy.load("en_core_web_sm")

def clean_and_lemmatize(text: str, remove_stopwords: bool = True) -> list[str]:
    doc = nlp(text)

    words = []
    for token in doc:
        # Skip punctuation, spaces, and optionally stopwords
        if token.is_punct or token.is_space:
            continue
        if remove_stopwords and token.is_stop:
            continue

        words.append(token.lemma_.lower())
    return words