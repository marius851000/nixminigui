use std::collections::BTreeMap;

//TODO: consider automatic usage of `` if there are multiple line
pub fn escape_string(entry: &str) -> String {
    let mut result = String::new();
    result.push('"');
    for char in entry.chars() {
        match char {
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            x => result.push(x),
        };
    }
    result.push('"');
    result
}

#[test]
fn test_escape_string() {
    assert_eq!(escape_string("hello, world"), "\"hello, world\"");
    assert_eq!(escape_string("line\njum\"p"), "\"line\\njum\\\"p\"")
}

pub fn generate_dict_from_btreemap(map: &BTreeMap<String, String>) -> String {
    let mut result = String::new();
    result.push('{');
    for (key, value) in map.iter() {
        result.push_str(&format!("\n{} = {};", key, value));
    }
    result.push('\n');
    result.push('}');
    result
}

pub fn to_nix_vec(list: &[String]) -> String {
    format!(
        "[ {}]",
        list.iter().fold(String::new(), |mut result, to_add| {
            result.push_str(&to_add);
            result.push(' ');
            result
        })
    )
}
