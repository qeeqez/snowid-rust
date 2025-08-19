use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create a custom configuration for many nodes (16 bits = 65,536 nodes)
    let config = SnowIDConfig::builder()
        .epoch(1577836800000)
        .node_bits(16) // 16 bits for node ID = 65,536 nodes
        .unwrap()
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

    println!("\nGenerated ID: {id}");
    println!("Components:");
    println!("  Timestamp: {ts} ms since epoch");
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
