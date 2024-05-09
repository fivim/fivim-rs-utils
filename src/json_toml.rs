extern crate serde_json;
extern crate toml;
use serde_json::{from_str, Map as JsonMap, Value as JsonValue};
use std::error::Error;
use toml::Value as TomlValue;

pub fn json_to_toml(json_str: &str) -> Result<String, Box<dyn Error>> {
    let json_value: JsonValue = from_str(json_str)?;
    let toml_value = toml::Value::try_from(&json_value)?;
    let res = toml::to_string_pretty(&toml_value)?;
    Ok(res)
}

pub fn toml_to_json(toml_str: &str) -> Result<String, Box<dyn Error>> {
    let toml_value: TomlValue = toml::from_str(&toml_str)?;
    let json_value = toml_value_to_json_value(toml_value);
    Ok(json_value.to_string())
}

pub fn toml_value_to_json_value(toml_value: TomlValue) -> JsonValue {
    match toml_value {
        TomlValue::String(s) => JsonValue::String(s),
        TomlValue::Integer(i) => JsonValue::Number(i.into()),
        TomlValue::Float(f) => JsonValue::Number(serde_json::Number::from_f64(f).unwrap()),
        TomlValue::Boolean(b) => JsonValue::Bool(b),
        TomlValue::Array(arr) => {
            let mut json_arr = Vec::new();
            for item in arr {
                json_arr.push(toml_value_to_json_value(item));
            }
            JsonValue::Array(json_arr)
        }
        TomlValue::Table(table) => {
            let mut json_map = JsonMap::new();
            for (key, value) in table {
                json_map.insert(key, toml_value_to_json_value(value));
            }
            JsonValue::Object(json_map)
        }
        _ => JsonValue::Null,
    }
}

#[cfg(test)]
mod test {
    use crate::json_toml::{json_to_toml, toml_to_json};

    #[test]
    fn test_main() {
        let json_str = r#"
        {
            "editorState": {
                "root": {
                    "children": [{
                        "children": [{
                            "detail": 0,
                            "format": 0,
                            "mode": "normal",
                            "style": "",
                            "text": "ghf",
                            "type": "text",
                            "version": 1
                        }],
                        "direction": "ltr",
                        "format": "",
                        "indent": 0,
                        "type": "paragraph",
                        "version": 1
                    }],
                    "direction": "ltr",
                    "format": "",
                    "indent": 0,
                    "type": "root",
                    "version": 1
                }
            }
        }    
    "#;

        let toml_str = json_to_toml(json_str).unwrap();

        println!("TOML:");
        println!(">>>\n{}\n<<<", toml_str);

        let parsed_json_str = toml_to_json(&toml_str).unwrap();

        println!("JSON:");
        println!(">>>\n{}\n<<<", parsed_json_str);
    }
}
