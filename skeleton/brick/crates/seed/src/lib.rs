//! Seed: a thin, sans-I/O decision core you compose.
//!
//! This crate is the curated public entrypoint. It carries no logic of its own: every item here
//! is a re-export of [`seed_contract`].

#![forbid(unsafe_code)]
#![no_std]

pub use seed_contract::{Decision, Verdict, decide};
