use regex::Regex;
use serde_path_to_error::Path;
use std::sync::LazyLock;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

static RE_MISSING_FIELD: LazyLock<Regex> = LazyLock::new(|| re(r"missing field `(.+?)`"));

static RE_INVALID_TYPE: LazyLock<Regex> =
    LazyLock::new(|| re(r"invalid type: (.+?), expected (.+)"));

static RE_INVALID_VALUE: LazyLock<Regex> =
    LazyLock::new(|| re(r"invalid value: (.+?), expected (.+)"));

static RE_UNKNOWN_FIELD: LazyLock<Regex> = LazyLock::new(|| re(r"unknown field `(.+?)`"));

static RE_DUPLICATE_FIELD: LazyLock<Regex> = LazyLock::new(|| re(r"duplicate field `(.+?)`"));

pub fn format_deserialization_error(path: &Path, raw_err: &str, source_name: &str) -> String {
    let err_msg = raw_err.split(" at line ").next().unwrap_or(raw_err);

    // 获取字段路径
    let field_path = path.to_string();
    let display_field = if field_path.is_empty() || field_path == "." {
        source_name.to_string()
    } else {
        field_path.trim_start_matches('.').to_string()
    };

    // 1. 处理缺失字段 (missing field `code`)
    if let Some(caps) = RE_MISSING_FIELD.captures(err_msg) {
        return format!("need {} field", &caps[1]);
    }

    // 2. 处理类型不匹配 (invalid type: number `123`, expected a string)
    if let Some(caps) = RE_INVALID_TYPE.captures(err_msg) {
        let expected = caps[2].trim().replace("a ", "").replace("an ", "");
        return format!("field '{}' must be a {}", display_field, expected);
    }

    // 3. 处理无效值 (例如：枚举成员不存在，或数值超出范围)
    if let Some(caps) = RE_INVALID_VALUE.captures(err_msg) {
        let expected = caps[2].trim().replace("a ", "").replace("an ", "");
        return format!(
            "field '{}' has invalid value, expected {}",
            display_field, expected
        );
    }

    // 4. 处理重复字段
    if let Some(caps) = RE_DUPLICATE_FIELD.captures(err_msg) {
        return format!("field '{}' is duplicated", &caps[1]);
    }

    // 5. 处理未知字段 (需配合 #[serde(deny_unknown_fields)])
    if let Some(caps) = RE_UNKNOWN_FIELD.captures(err_msg) {
        return format!("field '{}' is not allowed", &caps[1]);
    }

    // 6. 处理语法错误或 EOF
    if err_msg.contains("EOF") || err_msg.contains("predetermined") {
        return format!("invalid {} format", source_name);
    }

    // 7. 兜底处理：有些自定义错误可能直接就是 err_msg
    if err_msg.contains("custom error") {
        return err_msg.replace("custom error: ", "");
    }

    format!("invalid data at '{}': {}", display_field, err_msg)
}
