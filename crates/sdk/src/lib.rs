// OpenZax Skills SDK
// Foundation for skill development

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Placeholder for SDK functionality
/// Will be expanded in Phase 2
pub fn init() {
    println!("OpenZax SDK v{}", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
