use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use snowid::{BASE62_MAX_LEN, SnowID, base62_decode, base62_encode, base62_encode_array};
use std::hint::black_box;

// Common test values used across benchmarks
const TEST_VALUES: [u64; 5] = [
    1,            // Small number
    1000,         // Medium number
    1_000_000,    // Large number
    u64::MAX / 2, // Very large number
    u64::MAX,     // Maximum u64
];

pub fn id_generation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ID Generation Comparison");

    // Create generator once
    let generator = SnowID::new(1).unwrap();

    // Benchmark int64 generation
    group.bench_function("int64_generation", |b| {
        b.iter(|| black_box(generator.generate()));
    });

    // Benchmark base62 generation (with String allocation)
    group.bench_function("base62_generation_string", |b| {
        b.iter(|| black_box(generator.generate_base62()));
    });

    // Benchmark zero-allocation base62 generation
    group.bench_function("base62_generation_array", |b| {
        b.iter(|| black_box(generator.generate_base62_array()));
    });

    // Benchmark zero-allocation base62 with caller buffer
    group.bench_function("base62_generation_into", |b| {
        b.iter(|| {
            let mut buf = [0u8; BASE62_MAX_LEN];
            let (_, raw_id) = generator.generate_base62_into(&mut buf);
            black_box(raw_id)
        });
    });

    group.finish();
}

pub fn base62_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Encoding");

    for &value in &TEST_VALUES {
        // String-allocating version
        group.bench_with_input(
            BenchmarkId::new("base62_encode_string", value),
            &value,
            |b, &value| {
                b.iter(|| black_box(base62_encode(value)));
            },
        );

        // Zero-alloc array version
        group.bench_with_input(
            BenchmarkId::new("base62_encode_array", value),
            &value,
            |b, &value| {
                b.iter(|| black_box(base62_encode_array(value)));
            },
        );
    }

    group.finish();
}

pub fn base62_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Decoding");

    for &value in &TEST_VALUES {
        // Pre-encode the value for decoding benchmarks
        let encoded = base62_encode(value);

        group.bench_with_input(
            BenchmarkId::new("base62_decode", value),
            &encoded,
            |b, encoded| {
                b.iter(|| black_box(base62_decode(encoded).unwrap()));
            },
        );
    }

    group.finish();
}

pub fn roundtrip_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Base62 Roundtrip");

    for &value in &TEST_VALUES {
        group.bench_with_input(
            BenchmarkId::new("base62_roundtrip", value),
            &value,
            |b, &value| {
                b.iter(|| {
                    let encoded = base62_encode(value);
                    black_box(base62_decode(&encoded).unwrap());
                });
            },
        );
    }

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
