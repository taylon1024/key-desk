//! 变量的数据形状、校验，以及导出成 `.env` 文本。
//!
//! 写入数据库前走 `UpsertVariable::normalize`，读出来用 `Variable`。

use serde::{Deserialize, Serialize};

/// 已经落在数据库里的一条变量。
#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub id: i64,
    /// 变量名，例如 `DATABASE_URL`。
    pub key: String,
    /// 变量值。当前以明文保存。
    pub value: String,
    /// 分组名，例如 `dev`、`prod`。同一作用域内变量名不能重复。
    pub scope: String,
    /// 给人看的说明，导出时变成 `#` 注释。
    pub description: String,
    /// 只影响界面是否遮罩，不代表值已加密。
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 界面提交的原始输入。缺省字段在 `normalize` 里补上。
#[derive(Debug, Deserialize)]
pub struct UpsertVariable {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_secret: Option<bool>,
}

/// 通过校验、可以直接写入数据库的变量。
#[derive(Debug, Clone)]
pub struct NewVariable {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub description: String,
    pub is_secret: bool,
}

impl UpsertVariable {
    /// 去掉首尾空白，补上缺省的作用域和敏感标记，并校验名字。
    ///
    /// 作用域省略时用 `default`。失败时返回给界面展示的错误文案。
    pub fn normalize(self) -> Result<NewVariable, String> {
        let key = self.key.trim().to_string();
        validate_key(&key)?;

        let scope = self
            .scope
            .unwrap_or_else(|| "default".to_string())
            .trim()
            .to_string();
        validate_scope(&scope)?;
        validate_value(&self.value)?;

        Ok(NewVariable {
            key,
            value: self.value,
            scope,
            description: self.description.unwrap_or_default().trim().to_string(),
            is_secret: self.is_secret.unwrap_or(false),
        })
    }
}

/// 变量名规则与常见 shell 环境变量一致：字母或 `_` 开头，其余只能是字母、数字、`_`。
pub fn validate_key(key: &str) -> Result<(), String> {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return Err("name cannot be empty".to_string());
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err("name must start with a letter or underscore".to_string());
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("name can only contain letters, digits, and underscores".to_string());
    }
    Ok(())
}

const MAX_VALUE_LEN: usize = 4096;

/// 值只要求非空且不超过长度上限，不限制字符格式。
pub fn validate_value(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("value cannot be empty".to_string());
    }
    if value.chars().count() > MAX_VALUE_LEN {
        return Err(format!("value cannot exceed {MAX_VALUE_LEN} characters"));
    }
    Ok(())
}

/// 作用域不能为空，只允许字母、数字、`.`、`_`、`-`。
pub fn validate_scope(scope: &str) -> Result<(), String> {
    if scope.is_empty() {
        return Err("scope cannot be empty".to_string());
    }
    if !scope
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(
            "scope can only contain letters, digits, dots, underscores, and hyphens".to_string(),
        );
    }
    Ok(())
}

/// 剪贴板用的一行：`NAME=value`。值的引号规则与 `.env` 导出相同，不含备注。
pub fn format_assignment(variable: &Variable) -> String {
    format!("{}={}", variable.key, quote_dotenv_value(&variable.value))
}

/// 把变量排成 dotenv 文本。说明写在对应变量上方的 `#` 注释里。
pub fn format_dotenv(variables: &[Variable]) -> String {
    let mut out = String::new();
    for variable in variables {
        if !variable.description.is_empty() {
            for line in variable.description.lines() {
                out.push_str("# ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str(&variable.key);
        out.push('=');
        out.push_str(&quote_dotenv_value(&variable.value));
        out.push('\n');
    }
    out
}

/// 空值，或含空格、引号、换行和 `$` 等会干扰解析的字符时，用双引号包起来。
fn quote_dotenv_value(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\\' | '#' | '\'' | '$' | '`' | '='));
    if !needs_quotes {
        return value.to_string();
    }
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_env_keys() {
        assert!(validate_key("DATABASE_URL").is_ok());
        assert!(validate_key("_PRIVATE").is_ok());
    }

    #[test]
    fn accepts_any_api_key_text_within_length() {
        assert!(validate_value("sk-ant-api03/+=._~").is_ok());
        assert!(validate_value("任意格式的密钥 123").is_ok());
        assert!(validate_value("").is_err());
        assert!(validate_value(&"x".repeat(4097)).is_err());
    }

    #[test]
    fn rejects_invalid_env_keys() {
        assert!(validate_key("").is_err());
        assert!(validate_key("1PASSWORD").is_err());
        assert!(validate_key("MY-KEY").is_err());
    }

    #[test]
    fn quotes_values_that_need_escaping() {
        let variables = vec![Variable {
            id: 1,
            key: "GREETING".to_string(),
            value: "hello world".to_string(),
            scope: "default".to_string(),
            description: "sample".to_string(),
            is_secret: false,
            created_at: String::new(),
            updated_at: String::new(),
        }];
        let dotenv = format_dotenv(&variables);
        assert!(dotenv.contains("# sample\n"));
        assert!(dotenv.contains("GREETING=\"hello world\"\n"));
        assert_eq!(format_assignment(&variables[0]), "GREETING=\"hello world\"");
    }
}
