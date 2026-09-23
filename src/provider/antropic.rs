/// 新建 Anthropic 条目时的默认 Base URL（用户可改）。
pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";

/// 占位 Key，仅用于初始化表单（常见真 Key 以 `sk-ant-` 开头）。
pub const PLACEHOLDER_API_KEY: &str = "sk-ant-placeholder-replace-me";

#[derive(Debug, Clone)]
pub struct Anthropic {
    pub base_url: String,
    pub api_key: String,
}

impl Default for Anthropic {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl Anthropic {
    pub fn new() -> Self {
        Self::default()
    }
}
