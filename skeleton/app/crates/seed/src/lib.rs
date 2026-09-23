//! Seed: an application that composes sans-I/O bricks behind a thin I/O shell.
//!
//! [`domain`] is the functional core: pure decisions over explicit inputs. [`shell`] owns every
//! effect — standard input and output, and in a real application the filesystem, subprocesses,
//! and storage — and drives the core.

#![forbid(unsafe_code)]

pub mod domain;
pub mod shell;
