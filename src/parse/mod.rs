use serde::{Deserialize, Deserializer};

pub fn to_toml_ascii(string: &str) -> String {
    string
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub fn sanitize_string_toml_ascii<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(to_toml_ascii(s.as_str()))
}
