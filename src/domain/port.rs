//! # TCP Port Domain Model
//!
//! Type-safe representation of TCP port numbers.

/// A TCP port number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    /// The standard HTTPS port (443).
    pub const HTTPS: Self = Self(443);

    /// Returns the raw u16 representation.
    pub const fn code(self) -> u16 {
        self.0
    }
}

impl From<u16> for Port {
    fn from(p: u16) -> Self {
        Self(p)
    }
}

impl From<Port> for u16 {
    fn from(p: Port) -> Self {
        p.0
    }
}

impl Default for Port {
    /// Defaults to HTTPS port 443.
    fn default() -> Self {
        Self::HTTPS
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
