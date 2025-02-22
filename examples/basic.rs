use snowid::SnowID;

fn main() {
    // Create a generator with node ID 1
    let mut generator = SnowID::new(1).unwrap();

    // Generate some IDs
    let id1 = generator.generate();
    let id2 = generator.generate();
    let id3 = generator.generate();

    println!("Generated IDs (guaranteed to be monotonic):");
    println!("  ID1: {}", id1);
    println!("  ID2: {}", id2);
    println!("  ID3: {}", id3);

    // Extract components from an ID
    let (timestamp, node, sequence) = generator.extract.decompose(id1);
    println!("\nComponents of ID1:");
    println!("  Timestamp: {} ms since epoch", timestamp);
    println!("  Node ID: {}", node);
    println!("  Sequence: {}", sequence);

    // Or extract components individually
    let ts = generator.extract.timestamp(id2);
    let node = generator.extract.node(id2);
    let seq = generator.extract.sequence(id2);
    println!("\nComponents of ID2 (extracted individually):");
    println!("  Timestamp: {} ms since epoch", ts);
    println!("  Node ID: {}", node);
    println!("  Sequence: {}", seq);
} 