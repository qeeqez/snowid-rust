use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create a custom configuration for many nodes (12 bits = 4096 nodes)
    let config = SnowIDConfig::builder()
        .node_bits(12)  // 12 bits for node ID = 4096 nodes
        .build();

    // Create generator with node ID 42
    let mut generator = SnowID::with_config(42, config).unwrap();

    println!("Generator configuration:");
    println!("  Node bits: {}", generator.node_bits());
    println!("  Sequence bits: {}", generator.sequence_bits());
    println!("  Max node ID: {}", generator.max_node_id());
    println!("  Max sequence per ms: {}", generator.max_sequence());

    // Generate and analyze an ID
    let id = generator.generate();
    let (ts, node, seq) = generator.extract.decompose(id);

    println!("\nGenerated ID: {}", id);
    println!("Components:");
    println!("  Timestamp: {} ms since epoch", ts);
    println!("  Node ID: {} (of {})", node, generator.max_node_id());
    println!("  Sequence: {} (of {})", seq, generator.max_sequence());
} 