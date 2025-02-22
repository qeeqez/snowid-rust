use thiserror::Error;

/// Represents errors that can occur during SnowID operations
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SnowIDError {
    /// Error when node ID exceeds the maximum allowed value
    #[error("Node ID {node_id} is invalid. Maximum allowed value is {max}")]
    InvalidNodeId { node_id: u16, max: u16 },
    /// Error when clock moves backwards (system time issue)
    #[error("Clock moved backwards. Refusing to generate id for {delta} milliseconds")]
    ClockMovedBackwards { delta: i64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let invalid_node = SnowIDError::InvalidNodeId {
            node_id: 1024,
            max: 1023,
        };
        assert_eq!(
            invalid_node.to_string(),
            "Node ID 1024 is invalid. Maximum allowed value is 1023"
        );

        let clock_backwards = SnowIDError::ClockMovedBackwards { delta: 100 };
        assert_eq!(
            clock_backwards.to_string(),
            "Clock moved backwards. Refusing to generate id for 100 milliseconds"
        );
    }

    #[test]
    fn test_error_debug() {
        let invalid_node = SnowIDError::InvalidNodeId {
            node_id: 1024,
            max: 1023,
        };
        assert!(format!("{:?}", invalid_node).contains("InvalidNodeId"));
    }

    #[test]
    fn test_error_clone() {
        let original = SnowIDError::InvalidNodeId {
            node_id: 1024,
            max: 1023,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
