use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use std::thread;
use tsid_rust::TsidGenerator;

fn bench_single_thread_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Thread");
    let generator = TsidGenerator::new(1);

    group.bench_function("generate_id", |b| {
        b.iter(|| {
            black_box(generator.generate());
        });
    });

    group.bench_function("generate_100_sequential", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(generator.generate());
            }
        });
    });

    group.finish();
}

fn bench_concurrent_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent");
    
    // Test different numbers of threads
    for threads in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", threads), 
            threads,
            |b, &threads| {
                b.iter(|| {
                    let generator = Arc::new(TsidGenerator::new(1));
                    let mut handles = vec![];
                    
                    for _ in 0..threads {
                        let gen = generator.clone();
                        handles.push(thread::spawn(move || {
                            for _ in 0..100 {
                                black_box(gen.generate());
                            }
                        }));
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_component_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("Component Extraction");
    let generator = TsidGenerator::new(1);
    let tsid = generator.generate();

    group.bench_function("extract_components", |b| {
        b.iter(|| {
            black_box(tsid_rust::extract_from_tsid(black_box(tsid)));
        });
    });

    group.finish();
}

fn bench_multiple_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multiple Nodes");
    let generators: Vec<_> = (0..4).map(|i| TsidGenerator::new(i)).collect();

    group.bench_function("generate_across_nodes", |b| {
        b.iter(|| {
            for gen in &generators {
                black_box(gen.generate());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_thread_generation,
    bench_concurrent_generation,
    bench_component_extraction,
    bench_multiple_nodes
);
criterion_main!(benches);
