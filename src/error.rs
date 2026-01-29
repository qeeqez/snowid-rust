use std::error::Error;
use std::fmt;

/// Represents errors that can occur during SnowID operations
#[derive(Debug, Clone, PartialEq)]
pub enum SnowIDError {
    /// Error when node ID exceeds the maximum allowed value
    InvalidNodeId { node_id: u16, max: u16 },
}

impl fmt::Display for SnowIDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnowIDError::InvalidNodeId { node_id, max } => {
                write!(
                    f,
                    "Node ID {} is invalid. Maximum allowed value is {}",
                    node_id, max
                )
            }
        }
    }
}

impl Error for SnowIDError {}

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
    }

    #[test]
    fn test_error_debug() {
        let invalid_node = SnowIDError::InvalidNodeId {
            node_id: 1024,
            max: 1023,
        };
        assert!(format!("{invalid_node:?}").contains("InvalidNodeId"));
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
