use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create a custom configuration for many nodes (12 bits = 4096 nodes)
    let config = SnowIDConfig::builder()
        .epoch(1577836800000)
        .node_bits(16) // 12 bits for node ID = 4096 nodes
        .build();

    // Create generator with node ID 42
    let generator = SnowID::with_config(1, config).unwrap();

    println!("Generator configuration:");
    println!("  Node bits: {}", generator.config.node_bits());
    println!("  Sequence bits: {}", generator.config.sequence_bits());
    println!("  Max node ID: {}", generator.config.max_node_id());
    println!("  Max sequence ID: {}", generator.config.max_sequence_id());
    println!("  Epoch: {}", generator.config.epoch());

    // Generate and analyze an ID
    let id = generator.generate();
    let (ts, node, seq) = generator.extract.decompose(id);

    println!("\nGenerated ID: {}", id);
    println!("Components:");
    println!("  Timestamp: {} ms since epoch", ts);
    println!(
        "  Node ID: {} (of {})",
        node,
        generator.config.max_node_id()
    );
    println!(
        "  Sequence: {} (of {})",
        seq,
        generator.config.max_sequence_id()
    );
}
