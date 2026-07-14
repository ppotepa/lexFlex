use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verbosity {
    Compact,
    Normal,
    Detailed,
}

impl Verbosity {
    pub fn as_str(self) -> &'static str {
        match self {
            Verbosity::Compact => "compact",
            Verbosity::Normal => "normal",
            Verbosity::Detailed => "detailed",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Verbosity::Compact => Verbosity::Normal,
            Verbosity::Normal => Verbosity::Detailed,
            Verbosity::Detailed => Verbosity::Compact,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSettings {
    pub verbosity: Verbosity,
}

impl Default for ChatSettings {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Verbosity;

    #[test]
    fn verbosity_cycles() {
        assert_eq!(Verbosity::Compact.next(), Verbosity::Normal);
        assert_eq!(Verbosity::Normal.next(), Verbosity::Detailed);
        assert_eq!(Verbosity::Detailed.next(), Verbosity::Compact);
    }
}
