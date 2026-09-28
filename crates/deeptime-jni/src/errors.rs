// SPDX-License-Identifier: LGPL-3.0-or-later
//! Errors across the JNI boundary (plan §6.3): `BridgeError`, the custom jni-rs `ErrorPolicy` that
//! turns it (and any panic) into the Java exceptions of package
//! `com.minecraftindustrialcomplex.deeptime.nativebridge`, the panic hook that records where a panic
//! happened, and the process-wide native log ring.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::sync::{LazyLock, Once};

use deeptime_core::CoreError;
use deeptime_core::log::{Level, LogRing};
use jni::Env;
use jni::errors::ErrorPolicy;
use jni::strings::JNIString;

/// Binary-name prefix of the Java exception classes.
const PKG: &str = "com/minecraftindustrialcomplex/deeptime/nativebridge/";

/// Log lines not tied to a session (panics, create/destroy); `drainLog(0)` empties it.
pub static GLOBAL_LOG: LazyLock<LogRing> = LazyLock::new(LogRing::default);

/// Everything a native entry point can fail with.
#[derive(Debug)]
pub enum BridgeError {
    Core(CoreError),
    /// A JNI call failed; usually a Java exception is already pending and propagates as is.
    Jni(jni::errors::Error),
    /// No live session has this id (destroyed, or never created).
    UnknownSession(u64),
    /// A call on this session panicked earlier.
    Poisoned(u64),
    /// A panic, with its message and location.
    Panic(String),
    /// A bad argument from Java (null string, short array, wrong buffer).
    IllegalArgument(String),
}

impl From<jni::errors::Error> for BridgeError {
    fn from(e: jni::errors::Error) -> Self {
        BridgeError::Jni(e)
    }
}

impl From<CoreError> for BridgeError {
    fn from(e: CoreError) -> Self {
        BridgeError::Core(e)
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeError::Core(e) => write!(f, "{e}"),
            BridgeError::Jni(e) => write!(f, "JNI error: {e}"),
            BridgeError::UnknownSession(id) => {
                write!(
                    f,
                    "no session with id {id} (destroyed, stale or forged handle)"
                )
            }
            BridgeError::Poisoned(id) => {
                write!(
                    f,
                    "session {id} panicked earlier and is poisoned; destroy it"
                )
            }
            BridgeError::Panic(msg) => f.write_str(msg),
            BridgeError::IllegalArgument(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for BridgeError {}

impl BridgeError {
    /// The JNI binary name of the exception class Java sees (plan §6.3).
    pub fn java_class(&self) -> String {
        let own = |name: &str| format!("{PKG}{name}");
        match self {
            BridgeError::Core(CoreError::PresetInvalid { .. }) => own("PresetInvalidException"),
            BridgeError::Core(CoreError::MemoryCap { .. }) => own("MemoryCapException"),
            BridgeError::Core(CoreError::Io { .. }) => own("DeepTimeIoException"),
            BridgeError::Core(CoreError::Cancelled) => own("CancelledException"),
            BridgeError::Core(CoreError::Poisoned) | BridgeError::Poisoned(_) => {
                own("SessionPoisonedException")
            }
            BridgeError::Core(CoreError::InvalidState(_)) | BridgeError::UnknownSession(_) => {
                "java/lang/IllegalStateException".into()
            }
            BridgeError::Core(CoreError::Unsupported(_)) => {
                "java/lang/UnsupportedOperationException".into()
            }
            BridgeError::Panic(_) => own("NativePanicException"),
            BridgeError::IllegalArgument(_) => "java/lang/IllegalArgumentException".into(),
            BridgeError::Jni(_) => "java/lang/RuntimeException".into(),
        }
    }
}

/// The error policy of every entry point: throws the `BridgeError`'s Java exception, or
/// `NativePanicException` for a panic, and returns the type's default (`0`, `null`) to Java.
pub struct ThrowDeepTimeException;

impl<T: Default> ErrorPolicy<T, BridgeError> for ThrowDeepTimeException {
    type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

    fn on_error<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
        err: BridgeError,
    ) -> jni::errors::Result<T> {
        throw(env, &err.java_class(), &err.to_string());
        Ok(T::default())
    }

    fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
        payload: Box<dyn Any + Send + 'static>,
    ) -> jni::errors::Result<T> {
        let msg = panic_message(payload.as_ref());
        // Dropping a payload may itself panic; leak it rather than unwind here.
        if let Err(again) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(payload)))
        {
            std::mem::forget(again);
        }
        throw(env, &format!("{PKG}NativePanicException"), &msg);
        Ok(T::default())
    }
}

/// Throws `class(message)` unless an exception is already pending (which then wins: it is the
/// more precise one, e.g. an `OutOfMemoryError` from `NewStringUTF`).
fn throw(env: &mut Env<'_>, class: &str, message: &str) {
    if env.exception_check() {
        return;
    }
    // `throw_new` reports a successful throw as `Err(JavaException)`; a failed class lookup leaves
    // its own `NoClassDefFoundError` pending. Either way Java sees an exception.
    let _ = env.throw_new(JNIString::from(class), JNIString::from(message));
}

thread_local! {
    static LAST_PANIC: RefCell<Option<String>> = const { RefCell::new(None) };
}

static HOOK: Once = Once::new();

/// Installs, once per process, a panic hook that records the panic's message and location for the
/// thread (so `NativePanicException` can carry both) and logs it to [`GLOBAL_LOG`]. It replaces the
/// default hook, so nothing is printed to the JVM's stderr (unit tests keep the previous hook too,
/// so libtest still reports panic messages).
pub fn install_panic_hook() {
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if cfg!(test) {
                previous(info);
            }
            let msg = payload_str(info.payload()).unwrap_or("non-string panic payload");
            let text = match info.location() {
                Some(loc) => format!("{msg} (at {}:{}:{})", loc.file(), loc.line(), loc.column()),
                None => msg.to_string(),
            };
            let thread = std::thread::current();
            GLOBAL_LOG.push(
                Level::Error,
                format!(
                    "native panic on thread {}: {text}",
                    thread.name().unwrap_or("<unnamed>")
                ),
            );
            LAST_PANIC.with(|p| *p.borrow_mut() = Some(text));
        }));
    });
}

fn payload_str(payload: &(dyn Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&'static str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

/// The panic's message with its location, as recorded by the hook on this thread; falls back to
/// the payload alone when the hook did not run (a panic on another thread, or no hook).
pub fn panic_message(payload: &(dyn Any + Send)) -> String {
    let recorded = LAST_PANIC.with(|p| p.borrow_mut().take());
    let from_payload = payload_str(payload).unwrap_or("non-string panic payload");
    match recorded {
        Some(text) if text.starts_with(from_payload) => text,
        _ => from_payload.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_follow_the_plan() {
        let pkg = |n: &str| format!("{PKG}{n}");
        let cases: Vec<(BridgeError, String)> = vec![
            (
                CoreError::preset("$.dummy.steps", "missing").into(),
                pkg("PresetInvalidException"),
            ),
            (
                CoreError::MemoryCap { needed: 2, cap: 1 }.into(),
                pkg("MemoryCapException"),
            ),
            (
                CoreError::io("/x", std::io::Error::other("boom")).into(),
                pkg("DeepTimeIoException"),
            ),
            (CoreError::Cancelled.into(), pkg("CancelledException")),
            (BridgeError::Panic("p".into()), pkg("NativePanicException")),
            (BridgeError::Poisoned(3), pkg("SessionPoisonedException")),
            (CoreError::Poisoned.into(), pkg("SessionPoisonedException")),
            (
                BridgeError::UnknownSession(9),
                "java/lang/IllegalStateException".into(),
            ),
            (
                CoreError::Unsupported("sampleColumn").into(),
                "java/lang/UnsupportedOperationException".into(),
            ),
        ];
        for (err, class) in cases {
            assert_eq!(err.java_class(), class, "{err}");
        }
    }

    #[test]
    fn the_hook_records_location() {
        install_panic_hook();
        let payload = std::panic::catch_unwind(|| panic!("boom here")).unwrap_err();
        let msg = panic_message(payload.as_ref());
        assert!(msg.starts_with("boom here (at "), "{msg}");
        assert!(msg.contains("errors.rs"), "{msg}");
        assert!(
            GLOBAL_LOG
                .drain()
                .iter()
                .any(|l| l.starts_with("ERROR\tnative panic") && l.contains("boom here"))
        );
    }
}
