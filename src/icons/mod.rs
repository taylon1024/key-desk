//! Embedded brand icons for LLM providers (SVG bytes).
//!
//! Icons are sourced primarily from [`@lobehub/icons-static-svg`](https://github.com/lobehub/lobe-icons).
//! Filenames match `crate::provider` module names (e.g. `antropic.svg`).

#![allow(dead_code)]

/// Map a provider module / id name to its embedded SVG bytes.
///
/// Returns `None` if no icon is registered for `name` (case-insensitive).
pub fn icon_svg(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "antropic" | "anthropic" => Some(include_str!("antropic.svg")),
        "baichuan" => Some(include_str!("baichuan.svg")),
        "dashscope" | "qwen" => Some(include_str!("dashscope.svg")),
        "deepseek" => Some(include_str!("deepseek.svg")),
        "fireworks" => Some(include_str!("fireworks.svg")),
        "gemini" | "google" => Some(include_str!("gemini.svg")),
        "groq" => Some(include_str!("groq.svg")),
        "hunyuan" => Some(include_str!("hunyuan.svg")),
        "lingyi" | "yi" => Some(include_str!("lingyi.svg")),
        "minimax" => Some(include_str!("minimax.svg")),
        "mistral" => Some(include_str!("mistral.svg")),
        "modelscope" => Some(include_str!("modelscope.svg")),
        "moonshot" | "kimi" => Some(include_str!("moonshot.svg")),
        "openai" => Some(include_str!("openai.svg")),
        "openrouter" => Some(include_str!("openrouter.svg")),
        "qianfan" | "baidu" => Some(include_str!("qianfan.svg")),
        "siliconflow" | "siliconcloud" => Some(include_str!("siliconflow.svg")),
        "stepfun" => Some(include_str!("stepfun.svg")),
        "together" | "togetherai" => Some(include_str!("together.svg")),
        "typesafeai" | "typesafe" => Some(include_str!("typesafeai.svg")),
        "volcengine" | "doubao" => Some(include_str!("volcengine.svg")),
        "xai" | "grok" => Some(include_str!("xai.svg")),
        "zhipu" | "zhipuai" | "chatglm" => Some(include_str!("zhipu.svg")),
        _ => None,
    }
}

/// All known provider icon module names (matching on-disk `.svg` stems).
pub const PROVIDER_ICON_NAMES: &[&str] = &[
    "antropic",
    "baichuan",
    "dashscope",
    "deepseek",
    "fireworks",
    "gemini",
    "groq",
    "hunyuan",
    "lingyi",
    "minimax",
    "mistral",
    "modelscope",
    "moonshot",
    "openai",
    "openrouter",
    "qianfan",
    "siliconflow",
    "stepfun",
    "together",
    "typesafeai",
    "volcengine",
    "xai",
    "zhipu",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_icon_loads_nonempty_svg() {
        for name in PROVIDER_ICON_NAMES {
            let svg = icon_svg(name).unwrap_or_else(|| panic!("missing icon: {name}"));
            assert!(!svg.is_empty(), "{name} empty");
            assert!(
                svg.contains("<svg"),
                "{name} does not look like SVG"
            );
        }
    }
}
