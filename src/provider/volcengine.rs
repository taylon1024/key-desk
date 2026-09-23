/// 新建火山方舟 / 豆包条目时的默认 Base URL（OpenAI 兼容，Ark `/api/v3`）。
pub const DEFAULT_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";

// 暂无通用 Anthropic 兼容 Base URL：Coding Plan 用 `/api/coding`，Agent Plan 用 `/api/plan`（与上述 `/api/v3` 非同一产品）。

/// 占位 Key，仅用于初始化表单。
pub const PLACEHOLDER_API_KEY: &str = "sk-placeholder-replace-me";
