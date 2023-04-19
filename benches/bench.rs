use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unicode_intervals::{internals, UnicodeCategory, UnicodeCategorySet, UnicodeVersion};

fn version(c: &mut Criterion) {
    let version = black_box(UnicodeVersion::V15_0_0);
    c.bench_function("version - normalized_categories", |b| {
        b.iter(|| {
            let _ = version.normalized_categories();
        })
    });
}

fn intervals(c: &mut Criterion) {
    let string = black_box("abcdef0123456789");
    let long_string =
        black_box("zxcvbnm,./asdfghjkl;'qwertyuiopZXCVBNM<>?ASDFGHJKL:QWERTYUIOP{}0123456");
    c.bench_function("intervals - from_str short", |b| {
        b.iter(|| internals::intervals::from_str(string))
    });
    c.bench_function("intervals - from_str long", |b| {
        b.iter(|| internals::intervals::from_str(long_string))
    });
    let uppercase = UnicodeVersion::V15_0_0.intervals_for(UnicodeCategory::Lu);
    let lowercase = UnicodeVersion::V15_0_0.intervals_for(UnicodeCategory::Ll);
    c.bench_function("intervals - subtract", |b| {
        b.iter(|| internals::intervals::subtract(lowercase.to_vec(), uppercase))
    });
}

fn categories(c: &mut Criterion) {
    let all_categories = black_box(UnicodeCategorySet::all());
    c.bench_function("categories - set - display", |b| {
        b.iter(|| {
            let _ = all_categories.to_string();
        })
    });
    c.bench_function("categories - merge", |b| {
        b.iter(|| {
            let _ = internals::categories::merge(
                Some(all_categories),
                black_box(UnicodeCategory::Lu.into()),
            );
        })
    });
}

fn query(c: &mut Criterion) {
    let version = black_box(unicode_intervals::UnicodeVersion::V15_0_0);
    c.bench_function("query - intervals_for_set - empty", |b| {
        b.iter(|| {
            let _ =
                internals::query::intervals_for_set(version, black_box(UnicodeCategorySet::new()));
        })
    });
    c.bench_function("query - intervals_for_set - all", |b| {
        b.iter(|| {
            let _ =
                internals::query::intervals_for_set(version, black_box(UnicodeCategorySet::all()));
        })
    });
    c.bench_function("query - intervals_for_set - single large", |b| {
        b.iter(|| {
            let _ =
                internals::query::intervals_for_set(version, black_box(UnicodeCategory::Lu).into());
        })
    });
    c.bench_function("query - intervals_for_set - multiple", |b| {
        b.iter(|| {
            let _ = internals::query::intervals_for_set(
                version,
                black_box(UnicodeCategory::Lu | UnicodeCategory::M),
            );
        })
    });
    let exclude_categories = black_box(UnicodeCategory::Lu);
    let min_codepoint = black_box(Some(0));
    let max_codepoint = black_box(Some(128));
    c.bench_function("query - top level - only codepoints", |b| {
        b.iter(|| {
            let _ = version.query(
                None,
                exclude_categories,
                min_codepoint,
                max_codepoint,
                None,
                None,
            );
        })
    });
    c.bench_function("query - top level - exclude chars", |b| {
        b.iter(|| {
            let _ = version.query(
                None,
                exclude_categories,
                min_codepoint,
                max_codepoint,
                None,
                black_box(Some("A@т")),
            );
        })
    });
    c.bench_function("query - top level - include and exclude chars", |b| {
        b.iter(|| {
            let _ = version.query(
                None,
                exclude_categories,
                min_codepoint,
                max_codepoint,
                black_box(Some("0123456789")),
                black_box(Some("QWERTYUIOP")),
            );
        })
    });
    c.bench_function("query - top level - include only", |b| {
        b.iter(|| {
            let _ = version.query(UnicodeCategory::Pc, None, 0, 50, "abc", None);
        })
    });
}

criterion_group!(default, version, intervals, categories, query);
criterion_main!(default);
