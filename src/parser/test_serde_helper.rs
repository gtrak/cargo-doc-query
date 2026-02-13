#[test]
fn test_deserialize_with_stack() {
    use crate::parser::serde_helper::deserialize_with_stack;

    // Create deeply nested JSON (5000 levels deep)
    let mut json = String::from("null");
    for _ in 0..5000 {
        json = format!("[{}]", json);
    }

    // This should work with serde_stacker but fail with regular serde_json
    let result: Result<serde_json::Value, _> = deserialize_with_stack(&json);
    assert!(
        result.is_ok(),
        "Should parse deeply nested JSON with extended stack"
    );
}

#[test]
fn test_typenum_parses() {
    use crate::parser::serde_helper::deserialize_with_stack;
    use rustdoc_types::Crate;
    use std::fs;

    // Try to parse typenum.json which was causing recursion limit errors
    let json_path = std::path::PathBuf::from("../mcp-cli-rs/target/doc/typenum.json");

    if json_path.exists() {
        let json_str = fs::read_to_string(&json_path).expect("Failed to read typenum.json");
        let result: Result<Crate, _> = deserialize_with_stack(&json_str);
        assert!(
            result.is_ok(),
            "Should parse typenum.json without recursion limit: {:?}",
            result.err()
        );
    }
}
