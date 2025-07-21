use chrono::{DateTime, Utc};
use snowid::SnowID;

fn main() {
    // Create a generator with node ID 1
    let mut generator = SnowID::new(1).unwrap();

    // Generate some IDs
    let id1 = generator.generate_base62();
    let id2 = generator.generate_base62();
    let id3 = generator.generate_base62();

    println!("Generated Base62 IDs (guaranteed to be monotonic):");
    print_id(&id1, &mut generator);
    print_id(&id2, &mut generator);
    print_id(&id3, &mut generator);

    // Or extract components individually
    let (encoded, raw_id) = generator.generate_base62_with_raw();

    println!("\nComponents of an ID (extracted individually):");
    println!("  Base62 ID: {encoded}");
    println!("  Raw ID: {raw_id}");

    // Decode and extract components
    let decoded = generator.decode_base62(&encoded).unwrap();
    let ts = generator.extract.timestamp(decoded);
    let node = generator.extract.node(decoded);
    let seq = generator.extract.sequence(decoded);

    println!("  Timestamp: {ts} ms since epoch");
    println!("  Node ID: {node}");
    println!("  Sequence: {seq}");
}

fn print_id(id: &str, generator: &mut SnowID) {
    // Decode the base62 ID to get the raw u64 value
    let raw_id = generator.decode_base62(id).unwrap();

    // Extract components from the raw ID
    let (since_epoch, node, sequence) = generator.extract.decompose(raw_id);
    let timestamp: u64 = since_epoch + generator.config.epoch();
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp as i64).unwrap();

    println!("  ID: {id}, Raw: {raw_id}, Timestamp: {timestamp}, Human date: {datetime}, Node ID: {node}, Sequence: {sequence}");
}
