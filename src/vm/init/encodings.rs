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

pub(crate) const ENCODING_NAMES: [(&str, &str, bool); 115] = [
    ("UTF_8", "UTF-8", false),
    ("CESU_8", "CESU-8", false),
    ("US_ASCII", "US-ASCII", false),
    ("ASCII", "US-ASCII", false),
    ("ASCII_8BIT", "ASCII-8BIT", false),
    ("BINARY", "ASCII-8BIT", false),
    ("UTF_16", "UTF-16", true),
    ("UTF_16BE", "UTF-16BE", false),
    ("UTF_16LE", "UTF-16LE", false),
    ("UTF_32", "UTF-32", true),
    ("UTF_32BE", "UTF-32BE", false),
    ("UTF_32LE", "UTF-32LE", false),
    ("ISO_8859_1", "ISO-8859-1", false),
    ("ISO8859_1", "ISO-8859-1", false),
    ("ISO_8859_2", "ISO-8859-2", false),
    ("ISO8859_2", "ISO-8859-2", false),
    ("ISO_8859_3", "ISO-8859-3", false),
    ("ISO8859_3", "ISO-8859-3", false),
    ("ISO_8859_4", "ISO-8859-4", false),
    ("ISO8859_4", "ISO-8859-4", false),
    ("ISO_8859_5", "ISO-8859-5", false),
    ("ISO8859_5", "ISO-8859-5", false),
    ("ISO_8859_6", "ISO-8859-6", false),
    ("ISO8859_6", "ISO-8859-6", false),
    ("ISO_8859_7", "ISO-8859-7", false),
    ("ISO8859_7", "ISO-8859-7", false),
    ("ISO_8859_8", "ISO-8859-8", false),
    ("ISO8859_8", "ISO-8859-8", false),
    ("ISO_8859_9", "ISO-8859-9", false),
    ("ISO8859_9", "ISO-8859-9", false),
    ("ISO_8859_10", "ISO-8859-10", false),
    ("ISO8859_10", "ISO-8859-10", false),
    ("ISO_8859_11", "ISO-8859-11", false),
    ("ISO8859_11", "ISO-8859-11", false),
    ("ISO_8859_13", "ISO-8859-13", false),
    ("ISO8859_13", "ISO-8859-13", false),
    ("ISO_8859_14", "ISO-8859-14", false),
    ("ISO8859_14", "ISO-8859-14", false),
    ("ISO_8859_15", "ISO-8859-15", false),
    ("ISO8859_15", "ISO-8859-15", false),
    ("ISO_8859_16", "ISO-8859-16", false),
    ("ISO8859_16", "ISO-8859-16", false),
    ("EUC_JP", "EUC-JP", false),
    ("EUC_KR", "EUC-KR", false),
    ("EUC_TW", "EUC-TW", false),
    ("EUC_CN", "EUC-CN", false),
    ("Shift_JIS", "Shift_JIS", false),
    ("SHIFT_JIS", "Shift_JIS", false),
    ("Windows_31J", "Windows-31J", false),
    ("KOI8_R", "KOI8-R", false),
    ("KOI8_U", "KOI8-U", false),
    ("Big5", "Big5", false),
    ("BIG5", "Big5", false),
    ("Emacs_Mule", "Emacs-Mule", false),
    ("EMACS_MULE", "Emacs-Mule", false),
    ("GB18030", "GB18030", false),
    ("GBK", "GBK", false),
    ("IBM437", "IBM437", false),
    ("IBM037", "IBM037", false),
    ("Windows_1250", "Windows-1250", false),
    ("Windows_1251", "Windows-1251", false),
    ("CP1251", "Windows-1251", false),
    ("IBM866", "IBM866", false),
    ("MacJapanese", "MacJapanese", false),
    ("MacCyrillic", "macCyrillic", false),
    ("TIS_620", "TIS-620", false),
    ("CP949", "CP949", false),
    ("IBM737", "IBM737", false),
    ("IBM775", "IBM775", false),
    ("CP850", "CP850", false),
    ("IBM852", "IBM852", false),
    ("CP852", "CP852", false),
    ("IBM855", "IBM855", false),
    ("CP855", "CP855", false),
    ("IBM857", "IBM857", false),
    ("IBM860", "IBM860", false),
    ("IBM861", "IBM861", false),
    ("IBM862", "IBM862", false),
    ("IBM863", "IBM863", false),
    ("IBM864", "IBM864", false),
    ("IBM865", "IBM865", false),
    ("IBM869", "IBM869", false),
    ("Windows_1252", "Windows-1252", false),
    ("Windows_1253", "Windows-1253", false),
    ("Windows_1254", "Windows-1254", false),
    ("Windows_1255", "Windows-1255", false),
    ("Windows_1256", "Windows-1256", false),
    ("Windows_1257", "Windows-1257", false),
    ("Windows_1258", "Windows-1258", false),
    ("Windows_874", "Windows-874", false),
    ("GB1988", "GB1988", false),
    ("GB2312", "GB2312", false),
    ("GB12345", "GB12345", false),
    ("MacCentEuro", "macCentEuro", false),
    ("MacCroatian", "macCroatian", false),
    ("MacGreek", "macGreek", false),
    ("MacIceland", "macIceland", false),
    ("MacRoman", "macRoman", false),
    ("MacRomania", "macRomania", false),
    ("MacThai", "macThai", false),
    ("MacTurkish", "macTurkish", false),
    ("MacUkraine", "macUkraine", false),
    ("EucJP_ms", "eucJP-ms", false),
    ("CP51932", "CP51932", false),
    ("UTF8_MAC", "UTF8-MAC", false),
    ("IBM720", "IBM720", false),
    ("CP720", "CP720", false),
    ("MACCYRILLIC", "macCyrillic", false),
    // The dummy encodings: Ruby names them and tags strings with them, but
    // converts nothing through them.
    ("ISO_2022_JP", "ISO-2022-JP", true),
    ("ISO2022_JP", "ISO-2022-JP", true),
    ("ISO_2022_JP_2", "ISO-2022-JP-2", true),
    ("UTF_7", "UTF-7", true),
    ("CP50221", "CP50221", true),
    ("Stateless_ISO_2022_JP", "stateless-ISO-2022-JP", true),
    ("STATELESS_ISO_2022_JP", "stateless-ISO-2022-JP", true),
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
