//! 各厂商的**新建模板**（默认 Base URL、占位 Key），方便分类和导出。
//! 不负责代发 HTTP；用户拿去什么工具里用都可以。

pub mod antropic;
pub mod baichuan;
pub mod dashscope;
pub mod deepseek;
pub mod fireworks;
pub mod gemini;
pub mod groq;
pub mod hunyuan;
pub mod lingyi;
pub mod minimax;
pub mod mistral;
pub mod modelscope;
pub mod moonshot;
pub mod openai;
pub mod openrouter;
pub mod qianfan;
pub mod siliconflow;
pub mod stepfun;
pub mod together;
pub mod typesafeai;
pub mod volcengine;
pub mod xai;
pub mod zhipu;

#[derive(Clone, Copy)]
pub struct LlmTemplate {
    pub label: &'static str,
    pub api_key_name: &'static str,
    pub base_url_name: &'static str,
    pub base_url: &'static str,
    pub placeholder_key: &'static str,
}

pub fn llm_templates() -> &'static [LlmTemplate] {
    TEMPLATES
}

const TEMPLATES: &[LlmTemplate] = &[
    template(
        "Anthropic",
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_BASE_URL",
        antropic::DEFAULT_BASE_URL,
        antropic::PLACEHOLDER_API_KEY,
    ),
    template(
        "Baichuan",
        "BAICHUAN_API_KEY",
        "BAICHUAN_BASE_URL",
        baichuan::DEFAULT_BASE_URL,
        baichuan::PLACEHOLDER_API_KEY,
    ),
    template(
        "DashScope",
        "DASHSCOPE_API_KEY",
        "DASHSCOPE_BASE_URL",
        dashscope::DEFAULT_BASE_URL,
        dashscope::PLACEHOLDER_API_KEY,
    ),
    template(
        "DeepSeek",
        "DEEPSEEK_API_KEY",
        "DEEPSEEK_BASE_URL",
        deepseek::DEFAULT_BASE_URL,
        deepseek::PLACEHOLDER_API_KEY,
    ),
    template(
        "Fireworks",
        "FIREWORKS_API_KEY",
        "FIREWORKS_BASE_URL",
        fireworks::DEFAULT_BASE_URL,
        fireworks::PLACEHOLDER_API_KEY,
    ),
    template(
        "Gemini",
        "GEMINI_API_KEY",
        "GEMINI_BASE_URL",
        gemini::DEFAULT_BASE_URL,
        gemini::PLACEHOLDER_API_KEY,
    ),
    template(
        "Groq",
        "GROQ_API_KEY",
        "GROQ_BASE_URL",
        groq::DEFAULT_BASE_URL,
        groq::PLACEHOLDER_API_KEY,
    ),
    template(
        "Hunyuan",
        "HUNYUAN_API_KEY",
        "HUNYUAN_BASE_URL",
        hunyuan::DEFAULT_BASE_URL,
        hunyuan::PLACEHOLDER_API_KEY,
    ),
    template(
        "Lingyi",
        "LINGYI_API_KEY",
        "LINGYI_BASE_URL",
        lingyi::DEFAULT_BASE_URL,
        lingyi::PLACEHOLDER_API_KEY,
    ),
    template(
        "MiniMax",
        "MINIMAX_API_KEY",
        "MINIMAX_BASE_URL",
        minimax::DEFAULT_BASE_URL,
        minimax::PLACEHOLDER_API_KEY,
    ),
    template(
        "Mistral",
        "MISTRAL_API_KEY",
        "MISTRAL_BASE_URL",
        mistral::DEFAULT_BASE_URL,
        mistral::PLACEHOLDER_API_KEY,
    ),
    template(
        "ModelScope",
        "MODELSCOPE_API_KEY",
        "MODELSCOPE_BASE_URL",
        modelscope::DEFAULT_BASE_URL,
        modelscope::PLACEHOLDER_API_KEY,
    ),
    template(
        "Moonshot",
        "MOONSHOT_API_KEY",
        "MOONSHOT_BASE_URL",
        moonshot::DEFAULT_BASE_URL,
        moonshot::PLACEHOLDER_API_KEY,
    ),
    template(
        "OpenAI",
        "OPENAI_API_KEY",
        "OPENAI_BASE_URL",
        openai::DEFAULT_BASE_URL,
        openai::PLACEHOLDER_API_KEY,
    ),
    template(
        "OpenRouter",
        "OPENROUTER_API_KEY",
        "OPENROUTER_BASE_URL",
        openrouter::DEFAULT_BASE_URL,
        openrouter::PLACEHOLDER_API_KEY,
    ),
    template(
        "Qianfan",
        "QIANFAN_API_KEY",
        "QIANFAN_BASE_URL",
        qianfan::DEFAULT_BASE_URL,
        qianfan::PLACEHOLDER_API_KEY,
    ),
    template(
        "SiliconFlow",
        "SILICONFLOW_API_KEY",
        "SILICONFLOW_BASE_URL",
        siliconflow::DEFAULT_BASE_URL,
        siliconflow::PLACEHOLDER_API_KEY,
    ),
    template(
        "StepFun",
        "STEPFUN_API_KEY",
        "STEPFUN_BASE_URL",
        stepfun::DEFAULT_BASE_URL,
        stepfun::PLACEHOLDER_API_KEY,
    ),
    template(
        "Together",
        "TOGETHER_API_KEY",
        "TOGETHER_BASE_URL",
        together::DEFAULT_BASE_URL,
        together::PLACEHOLDER_API_KEY,
    ),
    template(
        "TypeSafeAI",
        "TYPESAFEAI_API_KEY",
        "TYPESAFEAI_BASE_URL",
        typesafeai::DEFAULT_BASE_URL,
        typesafeai::PLACEHOLDER_API_KEY,
    ),
    template(
        "Volcengine",
        "VOLCENGINE_API_KEY",
        "VOLCENGINE_BASE_URL",
        volcengine::DEFAULT_BASE_URL,
        volcengine::PLACEHOLDER_API_KEY,
    ),
    template(
        "xAI",
        "XAI_API_KEY",
        "XAI_BASE_URL",
        xai::DEFAULT_BASE_URL,
        xai::PLACEHOLDER_API_KEY,
    ),
    template(
        "Zhipu",
        "ZHIPU_API_KEY",
        "ZHIPU_BASE_URL",
        zhipu::DEFAULT_BASE_URL,
        zhipu::PLACEHOLDER_API_KEY,
    ),
];

const fn template(
    label: &'static str,
    api_key_name: &'static str,
    base_url_name: &'static str,
    base_url: &'static str,
    placeholder_key: &'static str,
) -> LlmTemplate {
    LlmTemplate {
        label,
        api_key_name,
        base_url_name,
        base_url,
        placeholder_key,
    }
}
