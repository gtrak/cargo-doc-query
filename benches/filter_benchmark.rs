use cargo_doc_query::types::filter::{FilterConfig, FilterEngine};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn create_test_config_simple() -> FilterConfig {
    FilterConfig::default()
        .with_include("std::*")
        .with_exclude("*::test*")
}

fn create_test_config_complex() -> FilterConfig {
    let mut config = FilterConfig::default();
    for i in 0..50 {
        config = config.with_include(format!("crate::module{}::*", i));
    }
    for i in 0..50 {
        config = config.with_exclude(format!("*::Test{}", i));
    }
    config = config
        .with_kind("struct")
        .with_crate("std")
        .with_visibility("pub");
    config
}

fn bench_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_compile");

    group.bench_function("simple_patterns", |b| {
        let config = create_test_config_simple();
        b.iter(|| FilterEngine::compile(black_box(&config)).unwrap());
    });

    group.bench_function("complex_patterns", |b| {
        let config = create_test_config_complex();
        b.iter(|| FilterEngine::compile(black_box(&config)).unwrap());
    });

    group.finish();
}

fn bench_match_single(c: &mut Criterion) {
    let config = create_test_config_simple();
    let engine = FilterEngine::compile(&config).unwrap();

    c.bench_function("match_single", |b| {
        b.iter(|| {
            engine.matches(
                black_box("std::vec::Vec"),
                black_box("struct"),
                black_box("std"),
                black_box("pub"),
            )
        });
    });
}

fn bench_match_many(c: &mut Criterion) {
    let config = create_test_config_complex();
    let engine = FilterEngine::compile(&config).unwrap();

    let paths: Vec<(&str, &str, &str, &str)> = vec![
        ("std::vec::Vec", "struct", "std", "pub"),
        ("std::string::String", "struct", "std", "pub"),
        ("std::collections::HashMap", "struct", "std", "pub"),
        ("crate::foo::Bar", "struct", "my_crate", "pub"),
        ("std::test::TestStruct", "struct", "std", "pub"),
    ];

    c.bench_function("match_100_items", |b| {
        b.iter(|| {
            for _ in 0..20 {
                // 20 * 5 = 100 items
                for (path, kind, crate_name, vis) in &paths {
                    engine.matches(
                        black_box(path),
                        black_box(kind),
                        black_box(crate_name),
                        black_box(vis),
                    );
                }
            }
        });
    });
}

fn bench_no_filter_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_overhead");

    let empty_config = FilterConfig::default();
    let empty_engine = FilterEngine::compile(&empty_config).unwrap();

    group.bench_function("empty_filter_check", |b| {
        b.iter(|| {
            empty_engine.matches(
                black_box("any::path"),
                black_box("struct"),
                black_box("any"),
                black_box("pub"),
            )
        });
    });

    group.finish();
}

fn bench_unicode_and_special_chars(c: &mut Criterion) {
    let config = FilterConfig::default().with_include("crate::*");
    let engine = FilterEngine::compile(&config).unwrap();

    c.bench_function("unicode_paths", |b| {
        b.iter(|| {
            engine.matches(
                black_box("crate::日本語"),
                black_box("fn"),
                black_box("crate"),
                black_box("pub"),
            )
        });
    });

    c.bench_function("special_regex_chars", |b| {
        let config2 = FilterConfig::default().with_include("crate::foo.*bar");
        let engine2 = FilterEngine::compile(&config2).unwrap();

        b.iter(|| {
            engine2.matches(
                black_box("crate::foo.bar"),
                black_box("fn"),
                black_box("crate"),
                black_box("pub"),
            )
        });
    });
}

criterion_group!(
    benches,
    bench_compile,
    bench_match_single,
    bench_match_many,
    bench_no_filter_overhead,
    bench_unicode_and_special_chars
);
criterion_main!(benches);
