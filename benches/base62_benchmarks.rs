use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use snowid::{base62_decode, base62_encode};
use snowid::{SnowID, SnowIDBase62};

pub fn id_generation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ID Generation Comparison");

    // Standard int64 SnowID
    let int_generator = SnowID::new(1).unwrap();

    // Base62 SnowID
    let base62_generator = SnowIDBase62::new(1).unwrap();

    // Benchmark int64 generation
    group.bench_function("int64_generation", |b| {
        b.iter(|| {
            black_box(int_generator.generate());
        });
    });

    // Benchmark base62 generation
    group.bench_function("base62_generation", |b| {
        b.iter(|| {
            black_box(base62_generator.generate());
        });
    });

    // Benchmark int64 generation + manual base62 encoding
    group.bench_function("int64_with_base62_encoding", |b| {
        b.iter(|| {
            let id = int_generator.generate();
            black_box(base62_encode(id));
        });
    });

    group.finish();
}

pub fn base62_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Encoding");

    // Test values of different magnitudes
    let test_values = [
        1u64,         // Small number
        1000u64,      // Medium number
        1_000_000u64, // Large number
        u64::MAX / 2, // Very large number
        u64::MAX,     // Maximum u64
    ];

    for &value in &test_values {
        group.bench_with_input(BenchmarkId::new("encode", value), &value, |b, &value| {
            b.iter(|| {
                black_box(base62_encode(black_box(value)));
            });
        });
    }

    group.finish();
}

pub fn base62_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Decoding");

    // Generate encoded values of different magnitudes
    let test_values = [
        1u64,         // Small number
        1000u64,      // Medium number
        1_000_000u64, // Large number
        u64::MAX / 2, // Very large number
        u64::MAX,     // Maximum u64
    ];

    let encoded_values: Vec<String> = test_values
        .iter()
        .map(|&value| base62_encode(value))
        .collect();

    for (i, encoded) in encoded_values.iter().enumerate() {
        let test_value = test_values[i];

        group.bench_with_input(
            BenchmarkId::new("decode", test_value),
            encoded,
            |b, encoded| {
                b.iter(|| {
                    black_box(base62_decode(black_box(encoded))).unwrap();
                });
            },
        );
    }

    group.finish();
}

pub fn roundtrip_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Roundtrip");

    let test_value = 12345678901234u64;

    group.bench_function("encode_decode", |b| {
        b.iter(|| {
            let encoded = base62_encode(black_box(test_value));
            black_box(base62_decode(&encoded)).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    id_generation_comparison,
    base62_encoding,
    base62_decoding,
    roundtrip_benchmark
);
criterion_main!(benches);
