pub mod i18n;
pub mod user;
pub mod validated_json;

use std::collections::HashMap;

pub fn format_validation_errors(
    errors: &validator::ValidationErrors,
) -> HashMap<String, Vec<String>> {
    errors
        .field_errors()
        .iter()
        .map(|(field, errs)| {
            let messages = errs
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{} is invalid", field))
                })
                .collect();
            (field.to_string(), messages)
        })
        .collect()
}
