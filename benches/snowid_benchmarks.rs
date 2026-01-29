use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use snowid::{SnowID, SnowIDConfig};
use std::hint::black_box;

pub fn node_bits_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Node Bits Comparison");

    // Test different node bit lengths
    // This affects the balance between max nodes (2^node_bits) and sequences per ms (2^sequence_bits)
    for &node_bits in &[6, 8, 10, 12, 14, 16] {
        let config = SnowIDConfig::builder()
            .node_bits(node_bits)
            .unwrap()
            .build();

        // Calculate theoretical limits for documentation
        let max_nodes = 2u32.pow(node_bits as u32);
        let sequence_bits = 22 - node_bits; // Total bits for node+sequence is fixed at 22
        let max_sequence = 2u32.pow(sequence_bits as u32);

        group.bench_function(
            format!("bits_{node_bits}_nodes_{max_nodes}_seq_{max_sequence}"),
            |b| {
                let generator = SnowID::with_config(1, config).unwrap();
                b.iter(|| {
                    black_box(generator.generate());
                });
            },
        );
    }

    group.finish();
}

pub fn overflow_stress_single_thread(c: &mut Criterion) {
    // Reduce sequence capacity per ms to 64 by using node_bits=16
    let cfg = SnowIDConfig::builder().node_bits(16).unwrap().build();
    let generator = SnowID::with_config(1, cfg).unwrap();

    let mut group = c.benchmark_group("Overflow SingleThread");
    for &batch in &[64usize, 128, 256, 512] {
        group.bench_function(format!("batch/{batch}"), |b| {
            b.iter_batched(
                || (),
                |_| {
                    // Generate a batch of IDs as fast as possible to trigger overflow waits
                    let mut last = 0u64;
                    for _ in 0..batch {
                        last = generator.generate();
                    }
                    black_box(last)
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

pub fn overflow_stress_concurrent_lockfree(c: &mut Criterion) {
    // node_bits=16 -> sequence capacity 64 per ms, easier to hit overflow
    let cfg = SnowIDConfig::builder().node_bits(16).unwrap().build();
    let mut group = c.benchmark_group("Overflow Concurrent");

    for &threads in &[2usize, 4, 8] {
        for &per_thread in &[64usize, 256] {
            group.bench_function(format!("threads/{threads}/per_thread/{per_thread}"), |b| {
                b.iter_batched(
                    || std::sync::Arc::new(SnowID::with_config(1, cfg).unwrap()),
                    |generator| {
                        let mut handles = Vec::with_capacity(threads);
                        for _ in 0..threads {
                            let generator_clone = std::sync::Arc::clone(&generator);
                            handles.push(std::thread::spawn(move || {
                                let mut last = 0u64;
                                for _ in 0..per_thread {
                                    last = generator_clone.generate();
                                }
                                last
                            }));
                        }
                        let mut acc = 0u64;
                        for h in handles {
                            acc ^= h.join().unwrap();
                        }
                        black_box(acc);
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }

    group.finish();
}

pub fn component_extraction_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Component Extraction");
    let generator = SnowID::new(1).unwrap();
    let snowid = generator.generate();

    group.bench_function("extract_components", |b| {
        b.iter(|| {
            black_box(generator.extract.decompose(black_box(snowid)));
        });
    });

    group.finish();
}

pub fn concurrent_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent LockFree");

    for &thread_count in &[2, 4, 8] {
        group.bench_function(format!("threads/{thread_count}"), |b| {
            b.iter_batched(
                || std::sync::Arc::new(SnowID::new(1).unwrap()),
                |generator| {
                    let mut handles = Vec::with_capacity(thread_count);

                    for _ in 0..thread_count {
                        let generator_clone = std::sync::Arc::clone(&generator);
                        handles.push(std::thread::spawn(move || {
                            // Direct lock-free access - no Mutex!
                            black_box(generator_clone.generate())
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    node_bits_comparison,
    concurrent_benchmarks,
    component_extraction_benchmarks,
    overflow_stress_single_thread,
    overflow_stress_concurrent_lockfree
);
criterion_main!(benches);
