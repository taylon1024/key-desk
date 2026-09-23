//! 16×16 black-and-white pixel icons for LLM providers.

pub fn icon_pixel_png(name: &str) -> Option<&'static [u8]> {
    match name.trim().to_ascii_lowercase().as_str() {
        "antropic" | "anthropic" => Some(include_bytes!("pixel/antropic.png")),
        "baichuan" => Some(include_bytes!("pixel/baichuan.png")),
        "dashscope" => Some(include_bytes!("pixel/dashscope.png")),
        "deepseek" => Some(include_bytes!("pixel/deepseek.png")),
        "fireworks" => Some(include_bytes!("pixel/fireworks.png")),
        "gemini" => Some(include_bytes!("pixel/gemini.png")),
        "groq" => Some(include_bytes!("pixel/groq.png")),
        "hunyuan" => Some(include_bytes!("pixel/hunyuan.png")),
        "lingyi" => Some(include_bytes!("pixel/lingyi.png")),
        "minimax" => Some(include_bytes!("pixel/minimax.png")),
        "mistral" => Some(include_bytes!("pixel/mistral.png")),
        "modelscope" => Some(include_bytes!("pixel/modelscope.png")),
        "moonshot" => Some(include_bytes!("pixel/moonshot.png")),
        "openai" => Some(include_bytes!("pixel/openai.png")),
        "openrouter" => Some(include_bytes!("pixel/openrouter.png")),
        "qianfan" => Some(include_bytes!("pixel/qianfan.png")),
        "siliconflow" => Some(include_bytes!("pixel/siliconflow.png")),
        "stepfun" => Some(include_bytes!("pixel/stepfun.png")),
        "together" => Some(include_bytes!("pixel/together.png")),
        "typesafeai" => Some(include_bytes!("pixel/typesafeai.png")),
        "volcengine" => Some(include_bytes!("pixel/volcengine.png")),
        "xai" => Some(include_bytes!("pixel/xai.png")),
        "zhipu" => Some(include_bytes!("pixel/zhipu.png")),
        _ => None,
    }
}

#[cfg(test)]
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
    fn every_named_icon_has_pixel_png() {
        for name in PROVIDER_ICON_NAMES {
            let png = icon_pixel_png(name).unwrap_or_else(|| panic!("missing pixel: {name}"));
            assert!(png.len() > 20, "{name} pixel png too small");
            assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "{name} not PNG");
        }
    }
}
