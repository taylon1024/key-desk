use serde::Serialize;

pub const DEFAULT_BASE_URL: &str = "https://api.gemini.com/v1";

pub const PLACEHOLDER_API_KEY: &str = "sk-placeholder-replace-me";

pub struct Gemini {
    pub base_url: String,
    pub api_key: String,
}

impl Default for Gemini {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl Gemini {
    pub fn new() -> Self {
        Self::default()
    }
}