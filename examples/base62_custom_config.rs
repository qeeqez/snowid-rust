use snowid::{SnowIDBase62, SnowIDConfig};

fn main() {
    // Create custom configuration
    let config = SnowIDConfig::builder()
        .epoch(1577836800000) // 2020-01-01 00:00:00 UTC
        .node_bits(8) // Supports 256 nodes
        .build();

    // Create base62 generator with custom config
    let gen = SnowIDBase62::with_config(42, config).unwrap();

    // Generate a few IDs
    for _ in 0..5 {
        let (encoded_id, raw_id) = gen.generate_with_raw();
        println!("Base62: {}, Raw: {}", encoded_id, raw_id);

        // Decompose to show the components
        let (timestamp, node, sequence) = gen.decompose(&encoded_id).unwrap();
        println!(
            "  → Timestamp: {}, Node: {} (expected 42), Sequence: {}",
            timestamp, node, sequence
        );
    }

    println!("\nConfiguration:");
    println!("  Max Node ID: {}", gen.snowid.config.max_node_id());
    println!("  Node Bits: {}", gen.snowid.config.node_bits());
    println!("  Max Sequence: {}", gen.snowid.config.max_sequence_id());
}
