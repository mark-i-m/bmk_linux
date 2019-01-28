use std::str::from_utf8;
use std::process::Command;

/// A typed thin wrapper around `pgrep`.
///
/// Returns either the PID of the process or `None`
pub fn pgrep(name: &str) -> Option<isize> {
    let output = Command::new("pgrep").arg(name).output().unwrap();
    if output.status.success() {
        let pid = from_utf8(&output.stdout).unwrap().trim().parse().unwrap();
        Some(pid)
    } else {
        None
    }
}
