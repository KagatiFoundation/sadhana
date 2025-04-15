use lazy_static::lazy_static;

lazy_static! {
    static ref A_SELECTOR: scraper::Selector = scraper::Selector::parse("a").unwrap();
    static ref P_SELECTOR: scraper::Selector = scraper::Selector::parse("p").unwrap();
    static ref TITLE_SELECTOR: scraper::Selector = scraper::Selector::parse("title").unwrap();
}

#[derive(Debug)]
pub struct HtmlDoc {
    pub title: String,
    pub url: String,
    text_content: Option<String>,
    html: Option<scraper::Html>
}

impl HtmlDoc {
    pub fn parse(url: String, html_str: String) -> Self {
        let document = scraper::Html::parse_document(&html_str);

        let title: String = document.select(&TITLE_SELECTOR).next()
                                .map(|elem| elem.inner_html())
                                .unwrap_or_else(|| "No Title".to_string());

        HtmlDoc {
            text_content: None,
            title,
            url,
            html: Some(document)
        }
    }

    pub fn text(&mut self) -> Option<&str> {
        if self.text_content.is_none() {
            let document = self.html.as_ref()?;
            let mut body = String::new();

            for p in document.select(&P_SELECTOR) {
                let txt = p.text().collect::<Vec<_>>().join(" ");
                body.push_str(&txt);
            }

            self.text_content = Some(body);
        }
    
        self.text_content.as_deref()
    }

    pub fn extract_links(&self) -> Vec<String> {
        if self.html.is_none() {
            return vec![];
        }

        self.html.as_ref().unwrap().select(&A_SELECTOR)
            .filter_map(|elem| elem.value().attr("href"))
            .map(|href| href.to_string())
            .collect()
    }

    pub fn preprocess<F>(&mut self, f: F) 
    where 
        F: FnOnce(&str) -> String
    {
        if let Some(ref txt) = self.text_content {
            self.text_content = Some(f(txt));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_html() -> String {
        r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <p>Hello, this is a test.</p>
                <p>More content here.</p>
                <a href="https://example.com">Example</a>
                <a href="/local">Local Link</a>
            </body>
        </html>
        "#.to_string()
    }

    #[test]
    fn test_parse_title() {
        let doc = HtmlDoc::parse("http://test.com".to_string(), sample_html());
        assert_eq!(doc.title, "Test Page");
    }

    #[test]
    fn test_extract_text() {
        let mut doc = HtmlDoc::parse("http://test.com".to_string(), sample_html());
        let text = doc.text().unwrap();
        assert!(text.contains("Hello, this is a test."));
        assert!(text.contains("More content here."));
    }

    #[test]
    fn test_extract_links() {
        let doc = HtmlDoc::parse("http://test.com".to_string(), sample_html());
        let links = doc.extract_links();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com".to_string()));
        assert!(links.contains(&"/local".to_string()));
    }

    #[test]
    fn test_preprocess() {
        let mut doc = HtmlDoc::parse("http://test.com".to_string(), sample_html());
        doc.text(); // generate text_content first
        doc.preprocess(|txt| txt.replace("test", "TEST"));
        assert!(doc.text_content.as_ref().unwrap().contains("TEST"));
    }

    #[test]
    fn test_missing_title() {
        let html = "<html><body><p>Hi</p></body></html>";
        let doc = HtmlDoc::parse("url".to_string(), html.to_string());
        assert_eq!(doc.title, "No Title");
    }

    #[test]
    fn test_missing_p_and_a_tags() {
        let html = "<html><head><title>Minimal</title></head><body></body></html>";
        let mut doc = HtmlDoc::parse("url".to_string(), html.to_string());
        assert_eq!(doc.text(), Some(""));
        assert_eq!(doc.extract_links().len(), 0);
    }
}