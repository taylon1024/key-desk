/// 新建 Moonshot / Kimi（月之暗面）条目时的默认 Base URL（OpenAI 兼容）。
pub const DEFAULT_BASE_URL: &str = "https://api.moonshot.cn/v1";

/// Anthropic 兼容协议 Base URL。
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.moonshot.cn/anthropic";

/// 占位 Key，仅用于初始化表单。
pub const PLACEHOLDER_API_KEY: &str = "sk-placeholder-replace-me";

#[derive(Debug, Clone)]
pub struct Moonshot {
    pub base_url: String,
    pub api_key: String,
}

impl Default for Moonshot {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl Moonshot {
    pub fn new() -> Self {
        Self::default()
    }
}
