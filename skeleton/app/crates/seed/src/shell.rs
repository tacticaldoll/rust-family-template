//! The imperative shell: it owns every effect and drives the functional core.

use std::io::{self, BufRead, Write};

use crate::domain::{self, Decision};

/// Read lines from `input`, decide each through the core, and write accepted content to `output`.
pub fn run(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    for line in input.lines() {
        if let Decision::Accept(content) = domain::decide(&line?) {
            writeln!(output, "{content}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_writes_only_accepted_lines() {
        let mut output = Vec::new();
        run("a\n\n b \n".as_bytes(), &mut output).expect("in-memory run succeeds");
        assert_eq!(output, b"a\nb\n");
    }
}
