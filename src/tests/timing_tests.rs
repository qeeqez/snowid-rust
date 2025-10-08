#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_wait_next_millis_progresses() {
        let generator = SnowID::new(1).unwrap();
        let from = generator.get_time_since_epoch();
        let next = generator.wait_next_millis(from, 1);
        assert!(next > from);
    }

    #[test]
    fn test_wait_next_millis_progresses_no_spin() {
        let cfg = SnowIDConfig::builder()
            .enable_spin(false)
            .spin_loops(0)
            .spin_yield_every(0)
            .build();
        let generator = SnowID::with_config(1, cfg).unwrap();
        let from = generator.get_time_since_epoch();
        let next = generator.wait_next_millis(from, 1);
        assert!(next > from);
    }
}
