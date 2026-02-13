use crate::cache::store::{CacheStore, SerializableIndex};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_cache_key_deterministic(manifest_path: String) {
        let cache_store = CacheStore::new().unwrap();
        let test_index = SerializableIndex {
            format_version: 1,
            cache_key: "test".to_string(),
            nodes: vec![],
            edges: vec![],
        };

        // Filter out problematic strings
        if manifest_path.is_empty() {
            return Ok(());
        }

        // Allow only safe characters: alphanumeric, underscore, dash, dot, colon
        if !manifest_path.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'
        }) {
            return Ok(());
        }

        let key1 = cache_store
            .save(&format!("test-{}", manifest_path), &test_index)
            .unwrap();
        let key2 = cache_store
            .save(&format!("test-{}", manifest_path), &test_index)
            .unwrap();

        prop_assert_eq!(key1, key2);
    }

    #[test]
    fn prop_cache_key_uniqueness(manifest_path: String, rustc_version: String) {
        let cache_store = CacheStore::new().unwrap();
        let test_index = SerializableIndex {
            format_version: 1,
            cache_key: "test".to_string(),
            nodes: vec![],
            edges: vec![],
        };

        // Filter out problematic strings
        if manifest_path.is_empty() || rustc_version.is_empty() {
            return Ok(());
        }

        // Allow only safe characters for both paths
        if !manifest_path.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'
        }) || !rustc_version.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '.'
        }) {
            return Ok(());
        }

        let key1 = cache_store
            .save(&format!("test-{}-{}", manifest_path, rustc_version), &test_index)
            .unwrap();

        let different_manifest = format!("different-{}", manifest_path);

        let key2 = cache_store
            .save(&format!("test-{}", different_manifest), &test_index)
            .unwrap();

        prop_assert_ne!(key1, key2);
    }

    #[test]
    fn prop_serialize_deserialize_string(s: String) {
        let bytes = postcard::to_stdvec(&s).unwrap();
        let deserialized: String = postcard::from_bytes(&bytes).unwrap();

        prop_assert_eq!(s, deserialized);
    }

    #[test]
    fn prop_roundtrip_u32(n: u32) {
        let serialized = postcard::to_stdvec(&n).unwrap();
        let deserialized: u32 = postcard::from_bytes(&serialized).unwrap();

        prop_assert_eq!(n, deserialized);
    }

    #[test]
    fn prop_roundtrip_bool(b: bool) {
        let serialized = postcard::to_stdvec(&b).unwrap();
        let deserialized: bool = postcard::from_bytes(&serialized).unwrap();

        prop_assert_eq!(b, deserialized);
    }
}
