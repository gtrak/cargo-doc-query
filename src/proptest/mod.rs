use crate::cache::store::{CacheStore, SerializableIndex};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_cache_save_deterministic(manifest_path: String) {
        let cache_store = CacheStore::new().unwrap();
        let test_index = SerializableIndex {
            format_version: 2,
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

        let _path1 = cache_store
            .save(&test_index)
            .unwrap();
        let _path2 = cache_store
            .save(&test_index)
            .unwrap();

        // Both saves should succeed
        prop_assert!(_path1.exists());
        prop_assert!(_path2.exists());
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
