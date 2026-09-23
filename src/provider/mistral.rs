/// 新建 Mistral 条目时的默认 Base URL（OpenAI 兼容）。
pub const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1";

/// 占位 Key，仅用于初始化表单。
pub const PLACEHOLDER_API_KEY: &str = "sk-placeholder-replace-me";

#[derive(Debug, Clone)]
pub struct Mistral {
    pub base_url: String,
    pub api_key: String,
}

impl Default for Mistral {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl Mistral {
    pub fn new() -> Self {
        Self::default()
    }
}
