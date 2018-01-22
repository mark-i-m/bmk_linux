//! Routines for parsing /proc/[pid]/stat

use std::path::PathBuf;

/// Used when choosing which process to get stats for.
pub enum Process {
    /// The currently process
    Current,

    /// The process with the given pid. This may require root priveleges.
    Pid(isize),
}

pub fn get_path(which: Process) -> PathBuf {
    match which {
        Process::Current => "/proc/self/stat".into(),
        Process::Pid(pid) => format!("/proc/{}/stat", pid).into(),
    }
}

parser! {
    struct ProcPidStat;

    get_path(pid: Process);

    pid: isize,
    comm: String,
    state: char,
    ppid: isize,
    pgrp: isize,
    session: isize,
    tty_nr: isize,
    tpgid: isize,
    flags: usize,
    minflt: usize,
    cminflt: usize,
    majflt: usize,
    cmajflt: usize,
    utime: usize,
    stime: usize,
    cutime: isize,
    cstime: isize,
    priority: isize,
    nice: isize,
    num_threads: isize,
    itrealvalue: isize,
    starttime: usize,
    vsize: usize,
    rss: isize,
    rsslim: usize,
    startcode: usize,
    endcode: usize,
    startstack: usize,
    kstkesp: usize,
    kstkeip: usize,
    signal: usize,
    blocked: usize,
    sigignore: usize,
    sigcatch: usize,
    wchan: usize,
    nswap: usize,
    cnswap: usize,
    exit_signal: isize,
    processor: isize,
    rt_priority: usize,
    policy: usize,
    delayacct_blkio_ticks: usize,
    guest_time: usize,
    cguest_time: isize,
    start_data: usize,
    end_data: usize,
    start_brk: usize,
    arg_start: usize,
    arg_end: usize,
    env_start: usize,
    env_end: usize,
    exit_code: isize,
}

#[test]
fn test_parse() {
    let _ = ProcPidStat::read(Process::Current).unwrap();
    // TODO: something more thorough
}
