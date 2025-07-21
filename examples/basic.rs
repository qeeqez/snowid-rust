use chrono::{DateTime, Utc};
use snowid::SnowID;

fn main() {
    // Create a generator with node ID 1
    let mut generator = SnowID::new(1).unwrap();

    // Generate some IDs
    let id1 = generator.generate();
    let id2 = generator.generate();
    let id3 = generator.generate();

    println!("Generated IDs (guaranteed to be monotonic):");
    print_id(id1, &mut generator);
    print_id(id2, &mut generator);
    print_id(id3, &mut generator);

    // Or extract components individually
    let ts = generator.extract.timestamp(id2);
    let node = generator.extract.node(id2);
    let seq = generator.extract.sequence(id2);
    println!("\nComponents of ID3 (extracted individually):");
    println!("  Timestamp: {} ms since epoch", (id1 >> 12) & 0x3FF);
    println!("  Timestamp: {ts} ms since epoch");
    println!("  Node ID: {node}");
    println!("  Sequence: {seq}");
}

fn print_id(id: u64, generator: &mut SnowID) {
    let (since_epoch, node, sequence) = generator.extract.decompose(id);
    let timestamp: u64 = since_epoch + generator.config.epoch();
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp as i64).unwrap();

    println!("  ID: {id}, Timestamp: {timestamp}, Human date: {datetime}, Node ID: {node}, Sequence: {sequence}");
}
