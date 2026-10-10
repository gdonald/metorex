// The encodings metorex names, and the flags a file call reads.

/// The encodings metorex names, as (constant, the name the encoding reports,
/// whether it is a dummy). Ruby spells most of the names by turning the
/// underscores into dashes, and the ones that break that rule carry their own
/// spelling here. A dummy encoding is one Ruby names and tags strings with but
/// cannot convert through.
/// The waiting, scheduling, and resource-limit settings the operating system
/// names by number, which Ruby carries as constants on Process.
/// The list is a slice rather than a sized array because the clocks a
/// platform keeps differ, so its length is settled by the platform being
/// built for.
pub(crate) const PROCESS_CONSTANTS: &[(&str, i64)] = &[
    ("WNOHANG", libc::WNOHANG as i64),
    ("WUNTRACED", libc::WUNTRACED as i64),
    ("PRIO_PROCESS", libc::PRIO_PROCESS as i64),
    ("PRIO_PGRP", libc::PRIO_PGRP as i64),
    ("PRIO_USER", libc::PRIO_USER as i64),
    ("RLIM_INFINITY", libc::RLIM_INFINITY as i64),
    // Both names stand for "no limit at all" on the systems that
    // carry them, which is the value RLIM_INFINITY holds.
    ("RLIM_SAVED_MAX", libc::RLIM_INFINITY as i64),
    ("RLIM_SAVED_CUR", libc::RLIM_INFINITY as i64),
    ("RLIMIT_CPU", libc::RLIMIT_CPU as i64),
    ("RLIMIT_FSIZE", libc::RLIMIT_FSIZE as i64),
    ("RLIMIT_DATA", libc::RLIMIT_DATA as i64),
    ("RLIMIT_STACK", libc::RLIMIT_STACK as i64),
    ("RLIMIT_CORE", libc::RLIMIT_CORE as i64),
    ("RLIMIT_AS", libc::RLIMIT_AS as i64),
    ("RLIMIT_MEMLOCK", libc::RLIMIT_MEMLOCK as i64),
    ("RLIMIT_NPROC", libc::RLIMIT_NPROC as i64),
    ("RLIMIT_NOFILE", libc::RLIMIT_NOFILE as i64),
    ("RLIMIT_RSS", libc::RLIMIT_RSS as i64),
    ("CLOCK_REALTIME", libc::CLOCK_REALTIME as i64),
    ("CLOCK_MONOTONIC", libc::CLOCK_MONOTONIC as i64),
    (
        "CLOCK_PROCESS_CPUTIME_ID",
        libc::CLOCK_PROCESS_CPUTIME_ID as i64,
    ),
    (
        "CLOCK_THREAD_CPUTIME_ID",
        libc::CLOCK_THREAD_CPUTIME_ID as i64,
    ),
    // The resource limits and clocks this platform keeps beyond the ones
    // every Unix has.
    #[cfg(target_os = "linux")]
    ("RLIMIT_MSGQUEUE", libc::RLIMIT_MSGQUEUE as i64),
    #[cfg(target_os = "linux")]
    ("RLIMIT_NICE", libc::RLIMIT_NICE as i64),
    #[cfg(target_os = "linux")]
    ("RLIMIT_RTPRIO", libc::RLIMIT_RTPRIO as i64),
    #[cfg(target_os = "linux")]
    ("RLIMIT_RTTIME", libc::RLIMIT_RTTIME as i64),
    #[cfg(target_os = "linux")]
    ("RLIMIT_SIGPENDING", libc::RLIMIT_SIGPENDING as i64),
    #[cfg(target_os = "linux")]
    ("CLOCK_MONOTONIC_RAW", libc::CLOCK_MONOTONIC_RAW as i64),
    #[cfg(target_os = "linux")]
    ("CLOCK_REALTIME_COARSE", libc::CLOCK_REALTIME_COARSE as i64),
    #[cfg(target_os = "linux")]
    (
        "CLOCK_MONOTONIC_COARSE",
        libc::CLOCK_MONOTONIC_COARSE as i64,
    ),
    #[cfg(target_os = "linux")]
    ("CLOCK_BOOTTIME", libc::CLOCK_BOOTTIME as i64),
    // The clocks this platform keeps beyond the four every Unix has.
    #[cfg(target_os = "macos")]
    ("CLOCK_MONOTONIC_RAW", libc::CLOCK_MONOTONIC_RAW as i64),
    #[cfg(target_os = "macos")]
    (
        "CLOCK_MONOTONIC_RAW_APPROX",
        libc::CLOCK_MONOTONIC_RAW_APPROX as i64,
    ),
    #[cfg(target_os = "macos")]
    ("CLOCK_UPTIME_RAW", libc::CLOCK_UPTIME_RAW as i64),
    #[cfg(target_os = "macos")]
    (
        "CLOCK_UPTIME_RAW_APPROX",
        libc::CLOCK_UPTIME_RAW_APPROX as i64,
    ),
];

/// The flags `File.open` accepts in `flags:`, and the ones a glob or fnmatch
/// is narrowed with. File, IO, and File::Constants all carry them.
pub(crate) const FILE_OPEN_FLAGS: [(&str, i64); 24] = [
    ("RDONLY", libc::O_RDONLY as i64),
    ("WRONLY", libc::O_WRONLY as i64),
    ("RDWR", libc::O_RDWR as i64),
    ("CREAT", libc::O_CREAT as i64),
    ("EXCL", libc::O_EXCL as i64),
    ("TRUNC", libc::O_TRUNC as i64),
    ("APPEND", libc::O_APPEND as i64),
    ("NONBLOCK", libc::O_NONBLOCK as i64),
    ("FNM_NOESCAPE", 0x01),
    ("FNM_PATHNAME", 0x02),
    ("FNM_DOTMATCH", 0x04),
    ("FNM_CASEFOLD", 0x08),
    ("FNM_EXTGLOB", 0x10),
    ("FNM_SYSCASE", 0),
    ("FNM_SHORTNAME", 0),
    ("NOCTTY", libc::O_NOCTTY as i64),
    ("SYNC", libc::O_SYNC as i64),
    ("DSYNC", libc::O_DSYNC as i64),
    ("NOFOLLOW", libc::O_NOFOLLOW as i64),
    // Nothing shares a file the way Windows does, so a file opened here is
    // already open the way `SHARE_DELETE` asks for.
    ("SHARE_DELETE", 0),
    ("LOCK_SH", libc::LOCK_SH as i64),
    ("LOCK_EX", libc::LOCK_EX as i64),
    ("LOCK_UN", libc::LOCK_UN as i64),
    ("LOCK_NB", libc::LOCK_NB as i64),
];
