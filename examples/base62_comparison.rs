use snowid::{SnowID, base62_encode};
use std::time::{Duration, Instant};

fn main() {
    // Create generators
    let int_gen = SnowID::new(1).unwrap();
    let base62_gen = SnowID::new(1).unwrap();

    // Number of IDs to generate for each test
    let iterations = 100_000;

    println!("Generating {iterations} IDs for each variant...\n");

    // Test int64 generation
    let start = Instant::now();
    let mut int_ids = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        int_ids.push(int_gen.generate());
    }
    let int_duration = start.elapsed();

    // Test base62 generation
    let start = Instant::now();
    let mut base62_ids = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        base62_ids.push(base62_gen.generate_base62());
    }
    let base62_duration = start.elapsed();

    // Test int64 + manual base62 encoding
    let start = Instant::now();
    let mut manual_base62_ids = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let id = int_gen.generate();
        manual_base62_ids.push(base62_encode(id));
    }
    let manual_base62_duration = start.elapsed();

    // Print results
    println!("Performance Results:");
    println!("--------------------");
    println!(
        "Int64 Generation:         {:?} ({:.2} ns/op)",
        int_duration,
        duration_to_ns(int_duration) / iterations as f64
    );

    println!(
        "Base62 Generation:        {:?} ({:.2} ns/op)",
        base62_duration,
        duration_to_ns(base62_duration) / iterations as f64
    );

    println!(
        "Int64 + Manual Base62:    {:?} ({:.2} ns/op)",
        manual_base62_duration,
        duration_to_ns(manual_base62_duration) / iterations as f64
    );

    // Calculate and print the percentage difference
    let int_ns = duration_to_ns(int_duration) / iterations as f64;
    let base62_ns = duration_to_ns(base62_duration) / iterations as f64;
    let percent_slower = ((base62_ns - int_ns) / int_ns) * 100.0;

    println!("\nBase62 is {percent_slower:.1}% slower than Int64");

    // Print ID examples
    println!("\nID Examples:");
    println!("------------");
    println!(
        "Int64:  {} ({} digits)",
        int_ids[0],
        int_ids[0].to_string().len()
    );
    println!("Base62: {} ({} chars)", base62_ids[0], base62_ids[0].len());
}

// Helper function to convert Duration to nanoseconds as f64
fn duration_to_ns(duration: Duration) -> f64 {
    duration.as_secs() as f64 * 1_000_000_000.0 + duration.subsec_nanos() as f64
}
