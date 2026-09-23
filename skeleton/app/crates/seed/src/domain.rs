//! The functional core: pure decisions over explicit inputs, with no I/O and no ambient clock.

/// The application's decision about one line of input.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The line carried content; holds it trimmed.
    Accept(String),
    /// The line was blank.
    Skip,
}

/// Decide what to do with one line of input.
pub fn decide(line: &str) -> Decision {
    match line.trim() {
        "" => Decision::Skip,
        content => Decision::Accept(content.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_are_skipped() {
        assert_eq!(decide("   "), Decision::Skip);
    }

    #[test]
    fn content_is_accepted_trimmed() {
        assert_eq!(decide("  hello "), Decision::Accept("hello".to_owned()));
    }
}
