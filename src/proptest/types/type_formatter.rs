use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_type_formatter_non_empty(type_str: String) {
        // Generate valid Rust type strings
        let valid_types: Vec<&str> = vec![
            "String", "Vec<String>", "Option<String>", "Result<String, String>",
            "HashMap<String, String>", "BTreeMap<String, String>", "VecDeque<String>",
            "HashSet<String>", "BinaryHeap<String>", "LinkedList<String>", "Vec<(String, i32)>",
            "Option<(String, i32)>", "Result<(String, i32), String>",
        ];

        if let Some(valid_type) = valid_types.iter().find(|t| t.starts_with(&type_str.chars().take(5).collect::<String>())) {
            let type_str = valid_type.to_string();
            let parsed = crate::query::format::parse_type(&type_str).unwrap();
            let formatted = crate::query::format::TypeFormatter::format_type(&parsed);

            prop_assert!(!formatted.is_empty());
            prop_assert!(formatted.contains(&type_str.chars().take(5).collect::<String>()));
        }
    }

    #[test]
    fn prop_type_formatter_basic_types() {
        let base_type in "\"String\" | \"i32\" | \"f64\" | \"bool\"";

        let parsed = crate::query::format::parse_type(base_type).unwrap();
        let formatted = crate::query::format::TypeFormatter::format_type(&parsed);

        prop_assert!(!formatted.is_empty());
        prop_assert!(formatted.contains("String") || formatted.contains("i32")
                    || formatted.contains("f64") || formatted.contains("bool"));
    }

    #[test]
    fn prop_type_formatter_generics() {
        let base_type in "\"Vec<T>\" | \"Option<T>\" | \"Result<T,E>\"";

        let parsed = crate::query::format::parse_type(base_type).unwrap();
        let formatted = crate::query::format::TypeFormatter::format_type(&parsed);

        prop_assert!(!formatted.is_empty());
    }

    #[test]
    fn prop_type_formatter_borrowed() {
        let base_type in "&str | &String | &Vec<i32>";

        let parsed = crate::query::format::parse_type(base_type).unwrap();
        let formatted = crate::query::format::TypeFormatter::format_type(&parsed);

        prop_assert!(!formatted.is_empty());
    }

    #[test]
    fn prop_type_formatter_ref() {
        let base_type in "&T | &mut T";

        let parsed = crate::query::format::parse_type(base_type).unwrap();
        let formatted = crate::query::format::TypeFormatter::format_type(&parsed);

        prop_assert!(!formatted.is_empty());
    }
}
