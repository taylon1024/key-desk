pub const PLACEHOLDER_API_KEY: &str = "apikey_placeholder-replace-me";
pub const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1";



#[derive(Debug, Clone)]
pub struct TypeSafeAI {
    pub base_url: String,
    pub api_key: String,
}

impl Default for TypeSafeAI {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl TypeSafeAI {
    pub fn new() -> Self {
        Self::default()
    }
}
