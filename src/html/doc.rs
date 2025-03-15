#[derive(Debug)]
pub struct HtmlDoc {
    pub title: String,
    pub url: String,
    pub content: String
}

impl HtmlDoc {
    pub fn new(title: String, url: String) -> Self {
        Self {
            title,
            url,
            content: String::from("")
        }
    }

    pub fn with_content(title: String, url: String, content: String) -> Self {
        Self {
            title,
            url,
            content
        }
    }

    pub fn proprocess(&mut self) {
        self.content = self.content.replace("\n", "");
    }
}