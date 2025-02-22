use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{Arc, Mutex};
use std::thread;
use tsid_rust::{Tsid, TsidConfig};

pub fn node_bits_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Node Bits Comparison");
    
    // Test different node bit lengths
    // This affects the balance between max nodes (2^node_bits) and sequences per ms (2^sequence_bits)
    for &node_bits in &[6, 8, 10, 12, 14, 16] {
        let config = TsidConfig::builder()
            .node_bits(node_bits)
            .build();
        
        // Calculate theoretical limits for documentation
        let max_nodes = 2u32.pow(node_bits as u32);
        let sequence_bits = 22 - node_bits; // Total bits for node+sequence is fixed at 22
        let max_sequence = 2u32.pow(sequence_bits as u32);
        
        group.bench_function(
            format!("bits_{}_nodes_{}_seq_{}", node_bits, max_nodes, max_sequence), 
            |b| {
                let mut generator = Tsid::with_config(1, config).unwrap();
                b.iter(|| {
                    black_box(generator.generate());
                });
            }
        );
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
                        black_box(gen.lock().unwrap().generate());
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

criterion_group!(
    benches,
    node_bits_comparison,
    concurrent_benchmarks,
    component_extraction_benchmarks
);
criterion_main!(benches);
