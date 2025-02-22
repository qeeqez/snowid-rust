use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{Arc, Mutex};
use std::thread;
use tsid_rust::Tsid;

pub fn single_thread_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Thread");
    let mut generator = Tsid::new(1).unwrap();

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

    group.bench_function("generate_1000_sequential", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(generator.generate());
            }
        });
    });

    group.bench_function("generate_10000_sequential", |b| {
        b.iter(|| {
            for _ in 0..10000 {
                black_box(generator.generate());
            }
        });
    });

    group.finish();
}

pub fn concurrent_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent");

    for &thread_count in &[2, 4, 8] {
        group.bench_function(format!("threads/{}", thread_count), |b| {
            b.iter(|| {
                let generator = Arc::new(Mutex::new(Tsid::new(1).unwrap()));
                let mut handles = Vec::with_capacity(thread_count);

                for _ in 0..thread_count {
                    let gen = Arc::clone(&generator);
                    handles.push(thread::spawn(move || {
                        for _ in 0..100 {
                            black_box(gen.lock().unwrap().generate());
                        }
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

pub fn component_extraction_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Component Extraction");
    let mut generator = Tsid::new(1).unwrap();
    let tsid = generator.generate();

    group.bench_function("extract_components", |b| {
        b.iter(|| {
            black_box(generator.extract.decompose(black_box(tsid)));
        });
    });

    group.finish();
}

pub fn multiple_nodes_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multiple Nodes");
    let mut generator = Tsid::new(1).unwrap();

    group.bench_function("generate_across_nodes", |b| {
        b.iter(|| {
            black_box(generator.generate());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    single_thread_benchmarks,
    concurrent_benchmarks,
    component_extraction_benchmarks,
    multiple_nodes_benchmarks
);
criterion_main!(benches);
