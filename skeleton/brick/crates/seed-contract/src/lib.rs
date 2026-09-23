//! The isolated core contract for Seed: a thin, sans-I/O decision core.
//!
//! The core owns one mechanism and no meaning. It exposes no `async fn`, reads no ambient clock,
//! and performs no I/O; a runtime drives it and supplies every input explicitly.

#![forbid(unsafe_code)]
#![no_std]

/// A domain-supplied verdict the core combines but never interprets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The domain certified the input as satisfied.
    Satisfied,
    /// The domain certified the input as unsatisfied.
    Unsatisfied,
}

/// The core's decision over a set of verdicts.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Every verdict was satisfied.
    Complete,
    /// At least one verdict was unsatisfied; carries how many remain.
    Pending(usize),
}

/// Decide over the domain's verdicts: complete when none remains unsatisfied.
pub fn decide(verdicts: &[Verdict]) -> Decision {
    match verdicts
        .iter()
        .filter(|verdict| **verdict == Verdict::Unsatisfied)
        .count()
    {
        0 => Decision::Complete,
        remaining => Decision::Pending(remaining),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_unsatisfied_verdict_is_complete() {
        assert_eq!(decide(&[]), Decision::Complete);
        assert_eq!(decide(&[Verdict::Satisfied]), Decision::Complete);
    }

    #[test]
    fn unsatisfied_verdicts_are_counted() {
        assert_eq!(
            decide(&[
                Verdict::Unsatisfied,
                Verdict::Satisfied,
                Verdict::Unsatisfied
            ]),
            Decision::Pending(2)
        );
    }
}
