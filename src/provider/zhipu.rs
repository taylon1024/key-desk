/// 新建智谱 GLM 条目时的默认 Base URL（OpenAI 兼容；国际站可用 `https://api.z.ai/api/paas/v4/`）。
pub const DEFAULT_BASE_URL: &str = "https://open.bigmodel.cn/api/paas/v4";

/// Anthropic 兼容协议 Base URL。
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://open.bigmodel.cn/api/anthropic";

/// 占位 Key，仅用于初始化表单。
pub const PLACEHOLDER_API_KEY: &str = "sk-placeholder-replace-me";

#[derive(Debug, Clone)]
pub struct Zhipu {
    pub base_url: String,
    pub api_key: String,
}

impl Default for Zhipu {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: PLACEHOLDER_API_KEY.to_string(),
        }
    }
}

impl Zhipu {
    pub fn new() -> Self {
        Self::default()
    }
}
