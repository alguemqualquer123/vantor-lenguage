pub struct Window {
    title: String,
    width: u32,
    height: u32,
}

impl Window {
    pub fn new() -> Self {
        Self {
            title: "Lexicon App".to_string(),
            width: 800,
            height: 600,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
