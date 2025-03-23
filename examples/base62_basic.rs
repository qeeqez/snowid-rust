use chrono::{DateTime, Utc};
use snowid::SnowIDBase62;

fn main() {
    // Create a generator with node ID 1
    let mut generator = SnowIDBase62::new(1).unwrap();

    // Generate some IDs
    let id1 = generator.generate();
    let id2 = generator.generate();
    let id3 = generator.generate();

    println!("Generated Base62 IDs (guaranteed to be monotonic):");
    print_id(&id1, &mut generator);
    print_id(&id2, &mut generator);
    print_id(&id3, &mut generator);

    // Or extract components individually
    let (encoded, raw_id) = generator.generate_with_raw();

    println!("\nComponents of an ID (extracted individually):");
    println!("  Base62 ID: {}", encoded);
    println!("  Raw ID: {}", raw_id);

    // Decode and extract components
    let decoded = generator.decode(&encoded).unwrap();
    let ts = generator.snowid.extract.timestamp(decoded);
    let node = generator.snowid.extract.node(decoded);
    let seq = generator.snowid.extract.sequence(decoded);

    println!("  Timestamp: {} ms since epoch", ts);
    println!("  Node ID: {}", node);
    println!("  Sequence: {}", seq);
}

fn print_id(id: &str, generator: &mut SnowIDBase62) {
    // Decode the base62 ID to get the raw u64 value
    let raw_id = generator.decode(id).unwrap();

    // Extract components from the raw ID
    let (since_epoch, node, sequence) = generator.snowid.extract.decompose(raw_id);
    let timestamp: u64 = since_epoch + generator.snowid.config.epoch();
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp as i64).unwrap();

    println!(
        "  ID: {}, Raw: {}, Timestamp: {}, Human date: {}, Node ID: {}, Sequence: {}",
        id, raw_id, timestamp, datetime, node, sequence
    );
}
