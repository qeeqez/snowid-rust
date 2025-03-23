use chrono::{DateTime, Utc};
use snowid::{SnowIDBase62, SnowIDConfig};

fn main() {
    // Create a custom configuration for many nodes (12 bits = 4096 nodes)
    let config = SnowIDConfig::builder()
        .epoch(1577836800000) // 2020-01-01 00:00:00 UTC
        .node_bits(16) // 16 bits for node ID = 65536 nodes
        .build();

    // Create generator with node ID 42
    let generator = SnowIDBase62::with_config(42, config).unwrap();

    println!("Base62 Generator configuration:");
    println!("  Node bits: {}", generator.snowid.config.node_bits());
    println!(
        "  Sequence bits: {}",
        generator.snowid.config.sequence_bits()
    );
    println!("  Max node ID: {}", generator.snowid.config.max_node_id());
    println!(
        "  Max sequence ID: {}",
        generator.snowid.config.max_sequence_id()
    );
    println!("  Epoch: {}", generator.snowid.config.epoch());

    // Generate and analyze an ID
    let encoded_id = generator.generate();
    let raw_id = generator.decode(&encoded_id).unwrap();
    let (ts, node, seq) = generator.snowid.extract.decompose(raw_id);

    // Calculate the actual timestamp
    let timestamp: u64 = ts + generator.snowid.config.epoch();
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp as i64).unwrap();

    println!("\nGenerated Base62 ID: {}", encoded_id);
    println!("Raw ID value: {}", raw_id);
    println!("Components:");
    println!("  Timestamp: {} ms since epoch", ts);
    println!("  Human date: {}", datetime);
    println!(
        "  Node ID: {} (of {})",
        node,
        generator.snowid.config.max_node_id()
    );
    println!(
        "  Sequence: {} (of {})",
        seq,
        generator.snowid.config.max_sequence_id()
    );

    // Generate a few more IDs to demonstrate monotonicity
    println!("\nGenerating a sequence of IDs:");
    for _ in 0..3 {
        let (encoded_id, raw_id) = generator.generate_with_raw();
        let (ts, node, seq) = generator.snowid.extract.decompose(raw_id);
        println!("  Base62: {}, Raw: {}", encoded_id, raw_id);
        println!("    → Timestamp: {}, Node: {}, Sequence: {}", ts, node, seq);
    }
}
