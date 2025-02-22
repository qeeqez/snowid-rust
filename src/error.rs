use std::fmt;

/// Represents errors that can occur during TSID operations
#[derive(Debug, Clone, PartialEq)]
pub enum TsidError {
    /// Error when node ID exceeds the maximum allowed value
    InvalidNodeId {
        node_id: u16,
        max_allowed: u16,
    },
    /// Error when clock moves backwards (system time issue)
    ClockBackwards,
    /// Error when sequence number overflows
    SequenceOverflow,
}

impl fmt::Display for TsidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TsidError::InvalidNodeId { node_id, max_allowed } => {
                write!(f, "Node ID {} exceeds maximum allowed value {}", node_id, max_allowed)
            }
            TsidError::ClockBackwards => write!(f, "System clock moved backwards"),
            TsidError::SequenceOverflow => write!(f, "Sequence number overflow"),
        }
    }
}

impl std::error::Error for TsidError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let invalid_node = TsidError::InvalidNodeId {
            node_id: 1024,
            max_allowed: 1023,
        };
        assert_eq!(
            invalid_node.to_string(),
            "Node ID 1024 exceeds maximum allowed value 1023"
        );

        let clock_backwards = TsidError::ClockBackwards;
        assert_eq!(clock_backwards.to_string(), "System clock moved backwards");

        let sequence_overflow = TsidError::SequenceOverflow;
        assert_eq!(sequence_overflow.to_string(), "Sequence number overflow");
    }

    #[test]
    fn test_error_debug() {
        let invalid_node = TsidError::InvalidNodeId {
            node_id: 1024,
            max_allowed: 1023,
        };
        assert!(format!("{:?}", invalid_node).contains("InvalidNodeId"));
    }

    #[test]
    fn test_error_clone() {
        let original = TsidError::InvalidNodeId {
            node_id: 1024,
            max_allowed: 1023,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
