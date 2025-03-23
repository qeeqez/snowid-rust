use snowid::SnowIDBase62;

fn main() {
    // Create a base62 generator with node ID 1
    let gen = SnowIDBase62::new(1).unwrap();

    // Generate a base62 encoded ID
    let encoded_id = gen.generate();
    println!("Base62 ID: {}", encoded_id); // Example: "2qPfVQh7Jw9"

    // Generate with raw value
    let (encoded_id, raw_id) = gen.generate_with_raw();
    println!("Base62: {}, Raw: {}", encoded_id, raw_id);

    // Decode a base62 ID back to u64
    let decoded = gen.decode(&encoded_id).unwrap();
    assert_eq!(decoded, raw_id);

    // Extract components from a base62 ID
    let (timestamp, node, sequence) = gen.decompose(&encoded_id).unwrap();
    println!(
        "Timestamp: {}, Node: {}, Sequence: {}",
        timestamp, node, sequence
    );
}
