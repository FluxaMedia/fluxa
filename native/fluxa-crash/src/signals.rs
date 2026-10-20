use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::OnceLock;

const FATAL: [libc::c_int; 5] = [
    libc::SIGSEGV,
    libc::SIGBUS,
    libc::SIGILL,
    libc::SIGFPE,
    libc::SIGABRT,
];

static MARKER: OnceLock<CString> = OnceLock::new();
static mut PREVIOUS: [Option<libc::sigaction>; 5] = [None; 5];

fn marker_path(dir: &Path) -> std::path::PathBuf {
    dir.join("native-crash")
}

extern "C" fn on_fatal(signal: libc::c_int) {
    if let Some(path) = MARKER.get() {
        let fd = unsafe {
            libc::open(
                path.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            )
        };
        if fd >= 0 {
            let digits = [b'0' + (signal / 10 % 10) as u8, b'0' + (signal % 10) as u8];
            unsafe {
                libc::write(fd, digits.as_ptr().cast(), digits.len());
                libc::close(fd);
            }
        }
    }
    if let Some(index) = FATAL.iter().position(|&known| known == signal) {
        unsafe {
            let previous = (&raw const PREVIOUS).cast::<Option<libc::sigaction>>().add(index);
            match *previous {
                Some(action) => {
                    libc::sigaction(signal, &action, std::ptr::null_mut());
                }
                None => {
                    libc::signal(signal, libc::SIG_DFL);
                }
            }
        }
    }
    unsafe {
        libc::raise(signal);
    }
}

pub(crate) fn arm(dir: &Path) {
    let Ok(path) = CString::new(marker_path(dir).as_os_str().as_bytes()) else {
        return;
    };
    if MARKER.set(path).is_err() {
        return;
    }
    for (index, signal) in FATAL.into_iter().enumerate() {
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = on_fatal as extern "C" fn(libc::c_int) as libc::sighandler_t;
            libc::sigemptyset(&mut action.sa_mask);
            let mut previous: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(signal, &action, &mut previous) == 0 {
                let slot = (&raw mut PREVIOUS).cast::<Option<libc::sigaction>>().add(index);
                *slot = Some(previous);
            }
        }
    }
}

pub(crate) fn report_previous(dir: &Path) {
    let path = marker_path(dir);
    let Ok(body) = std::fs::read_to_string(&path) else {
        return;
    };
    let _ = std::fs::remove_file(&path);
    let signal = body.trim().trim_start_matches('0').to_owned();
    sentry::with_scope(
        |scope| scope.set_tag("crash.signal", &signal),
        || {
            sentry::capture_message(
                &format!("native crash in previous run (signal {signal})"),
                sentry::Level::Fatal,
            )
        },
    );
    crate::flush();
}
