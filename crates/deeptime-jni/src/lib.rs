// SPDX-License-Identifier: LGPL-3.0-or-later
//! The JNI bridge (plan §6): every native method of
//! `com.minecraftindustrialcomplex.deeptime.nativebridge.DeepTimeNative`, the session handle table
//! and the panic guards. The only crate that depends on `jni`.
//!
//! Every entry point runs inside `EnvUnowned::with_env`, which catches unwinding panics, and
//! resolves with [`errors::ThrowDeepTimeException`], which maps a [`errors::BridgeError`] or a
//! panic to a Java exception (plan §6.3). Calls on a session additionally run under their own
//! `catch_unwind`, so a panic poisons only that session.

pub mod alloc;
pub mod errors;
pub mod handles;
pub mod selftest;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use deeptime_core::log::Level;
use deeptime_core::session::{PROGRESS_FIELDS, Session, SessionOptions};
use deeptime_core::{CoreError, FORMAT_VERSION};
use jni::objects::{JByteBuffer, JClass, JLongArray, JString};
use jni::sys::{jint, jlong};
use jni::{Env, EnvUnowned};

use crate::errors::{BridgeError, GLOBAL_LOG, ThrowDeepTimeException};
use crate::handles::{SESSIONS, SessionEntry};

pub use deeptime_core::VERSION;

/// The Rust target triple this library was built for.
pub const TARGET: &str = env!("DEEPTIME_TARGET");
/// The git commit it was built from (informational; never compared).
pub const GIT_SHA: &str = env!("DEEPTIME_GIT_SHA");

/// `version()`'s string: `<semver>+<git sha>;format=<f>;target=<rust triple>` (plan §6.1).
pub fn version_string() -> String {
    format!("{VERSION}+{GIT_SHA};format={FORMAT_VERSION};target={TARGET}")
}

type BridgeResult<T> = Result<T, BridgeError>;

/// Runs `f` with the panic hook installed, catching panics, and resolves errors to Java exceptions.
fn guard<'l, T, F>(env: &mut EnvUnowned<'l>, f: F) -> T
where
    T: Default + 'l,
    F: FnOnce(&mut Env<'l>) -> BridgeResult<T>,
{
    errors::install_panic_hook();
    env.with_env(f).resolve::<ThrowDeepTimeException>()
}

/// Looks the session up and runs `f` on it under `catch_unwind`: a panic poisons the session
/// (every later call throws `SessionPoisonedException`) and surfaces as `NativePanicException`.
fn with_session<R>(id: jlong, f: impl FnOnce(&Session) -> BridgeResult<R>) -> BridgeResult<R> {
    let entry = SESSIONS.get(id as u64)?;
    if entry.is_poisoned() {
        return Err(BridgeError::Poisoned(id as u64));
    }
    match catch_unwind(AssertUnwindSafe(|| f(&entry.session))) {
        Ok(result) => result,
        Err(payload) => {
            entry.poison();
            let msg = errors::panic_message(payload.as_ref());
            entry.session.log().push(
                Level::Error,
                format!("session {id} poisoned by a native panic: {msg}"),
            );
            Err(BridgeError::Panic(msg))
        }
    }
}

fn string_arg(env: &Env<'_>, s: &JString<'_>, name: &str) -> BridgeResult<String> {
    if s.is_null() {
        return Err(BridgeError::IllegalArgument(format!("{name} is null")));
    }
    Ok(s.try_to_string(env)?)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_version<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
) -> JString<'l> {
    guard(&mut env, |env| Ok(env.new_string(version_string())?))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_selfTest<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
) -> jlong {
    guard(&mut env, |_env| Ok(selftest::run()? as jlong))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_estimate<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    preset_json: JString<'l>,
    content_json: JString<'l>,
) -> JString<'l> {
    guard(&mut env, |env| {
        let preset = string_arg(env, &preset_json, "presetJson")?;
        let content = string_arg(env, &content_json, "contentJson")?;
        let json = deeptime_core::session::estimate(&preset, &content)?;
        Ok(env.new_string(json)?)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_create<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    preset_json: JString<'l>,
    content_json: JString<'l>,
    out_dir: JString<'l>,
    threads: jint,
    mem_cap_bytes: jlong,
) -> jlong {
    guard(&mut env, |env| {
        let preset = string_arg(env, &preset_json, "presetJson")?;
        let content = string_arg(env, &content_json, "contentJson")?;
        let out_dir = string_arg(env, &out_dir, "outDir")?;
        if threads < 0 {
            return Err(BridgeError::IllegalArgument(format!(
                "threads must be ≥ 0 (0 = all processors), got {threads}"
            )));
        }
        if mem_cap_bytes < 0 {
            return Err(BridgeError::IllegalArgument(format!(
                "memCapBytes must be ≥ 0 (0 = default), got {mem_cap_bytes}"
            )));
        }
        let session = Session::create(
            &preset,
            &content,
            SessionOptions {
                out_dir: PathBuf::from(out_dir),
                threads: threads as usize,
                mem_cap: mem_cap_bytes as u64,
            },
            Arc::new(alloc::AllocGauge),
        )?;
        let id = SESSIONS.insert(SessionEntry::new(session));
        GLOBAL_LOG.push(Level::Debug, format!("session {id} created"));
        Ok(id as jlong)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_advance<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    budget_millis: jint,
) -> jint {
    guard(&mut env, |_env| {
        with_session(session, |s| {
            let budget = Duration::from_millis(budget_millis.max(0) as u64);
            Ok(s.advance(budget)? as jint)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_cancel<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
) {
    guard(&mut env, |_env| {
        with_session(session, |s| {
            s.cancel();
            Ok(())
        })
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_progress<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    out: JLongArray<'l>,
) {
    guard(&mut env, |env| {
        if out.is_null() {
            return Err(BridgeError::IllegalArgument("out is null".into()));
        }
        let len = out.len(env)?;
        if len < PROGRESS_FIELDS {
            return Err(BridgeError::IllegalArgument(format!(
                "out has {len} elements, progress needs {PROGRESS_FIELDS}"
            )));
        }
        let values = with_session(session, |s| Ok(s.progress().to_array()))?;
        out.set_region(env, 0, &values)?;
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_keyframeCount<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
) -> jint {
    guard(&mut env, |_env| {
        with_session(session, |s| Ok(s.keyframe_count() as jint))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_keyframeInfo<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    index: jint,
) -> JString<'l> {
    guard(&mut env, |env| {
        let index = non_negative(index, "index")?;
        let json = with_session(session, |s| Ok(s.keyframe_info(index)?))?;
        Ok(env.new_string(json)?)
    })
}

fn non_negative(v: jint, name: &str) -> BridgeResult<u32> {
    u32::try_from(v)
        .map_err(|_| BridgeError::IllegalArgument(format!("{name} must be ≥ 0, got {v}")))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_copyKeyframe<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    index: jint,
    layer: jint,
    dst_rgba: JByteBuffer<'l>,
) {
    guard(&mut env, |env| {
        let index = non_negative(index, "index")?;
        let layer = non_negative(layer, "layer")?;
        if dst_rgba.is_null() {
            return Err(BridgeError::IllegalArgument("dstRgba is null".into()));
        }
        let not_direct = |_| BridgeError::IllegalArgument("dstRgba is not a direct buffer".into());
        let capacity = env
            .get_direct_buffer_capacity(&dst_rgba)
            .map_err(not_direct)?;
        let address = env
            .get_direct_buffer_address(&dst_rgba)
            .map_err(not_direct)?;
        if address.is_null() {
            return Err(BridgeError::IllegalArgument(
                "dstRgba is not a direct buffer".into(),
            ));
        }
        // SAFETY: a direct buffer's memory is `capacity` bytes at `address` and stays valid while
        // Java holds `dst_rgba`, which it does for the whole call; no other Rust reference to it
        // exists.
        let dst = unsafe { std::slice::from_raw_parts_mut(address, capacity) };
        with_session(session, |s| Ok(s.copy_keyframe(index, layer, dst)?))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_eventsSince<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    cursor: jlong,
    max: jint,
) -> JString<'l> {
    guard(&mut env, |env| {
        let json = with_session(session, |s| {
            Ok(s.events_since(cursor.max(0) as u64, max.max(0) as u32))
        })?;
        Ok(env.new_string(json)?)
    })
}

/// Session `0` drains the process-wide ring (panics, create/destroy); any other id drains that
/// session's ring, even when it is poisoned (the panic's own log line is there). Lines are
/// `LEVEL<TAB>message`, joined by `\n`; the empty string means nothing new.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_drainLog<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
) -> JString<'l> {
    guard(&mut env, |env| {
        let lines = if session == 0 {
            GLOBAL_LOG.drain()
        } else {
            SESSIONS.get(session as u64)?.session.log().drain()
        };
        Ok(env.new_string(lines.join("\n"))?)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_finalizeWrite<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
) -> JString<'l> {
    guard(&mut env, |env| {
        let manifest = with_session(session, |s| Ok(s.finalize_write()?))?;
        Ok(env.new_string(manifest)?)
    })
}

/// Removes the session from the table and cancels it. A caller still inside `advance` on another
/// thread keeps its `Arc`, so the session (and its Rayon pool) is freed when that call returns.
/// Works on poisoned sessions too.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_destroy<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
) {
    guard(&mut env, |_env| {
        let entry = SESSIONS.remove(session as u64)?;
        entry.session.cancel();
        drop(entry);
        GLOBAL_LOG.push(Level::Debug, format!("session {session} destroyed"));
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_sampleColumn<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    _world_data_dir: JString<'l>,
    _x: jint,
    _z: jint,
) -> JString<'l> {
    guard(&mut env, |_env| {
        Err(CoreError::Unsupported("sampleColumn (until P1-08)").into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string_has_the_documented_shape() {
        let v = version_string();
        let (semver_sha, rest) = v.split_once(';').unwrap();
        assert!(semver_sha.starts_with("0.0.1+"), "{v}");
        assert!(rest.starts_with("format=0.0;target="), "{v}");
        assert!(rest.ends_with(TARGET), "{v}");
    }

    #[test]
    fn a_panic_in_a_session_call_poisons_only_that_session() {
        errors::install_panic_hook();
        let make = |json: &str| {
            let s = Session::create(
                json,
                "",
                SessionOptions {
                    out_dir: std::env::temp_dir().join("deeptime-jni-poison-test"),
                    threads: 1,
                    mem_cap: 0,
                },
                Arc::new(alloc::AllocGauge),
            )
            .unwrap();
            SESSIONS.insert(SessionEntry::new(s)) as jlong
        };
        let bad = make(r#"{"dummy":{"steps":2,"sleep_ms":0,"panic_at_step":0}}"#);
        let good = make(r#"{"dummy":{"steps":2,"sleep_ms":0}}"#);
        let r = with_session(bad, |s| Ok(s.advance(Duration::MAX)?));
        match r {
            Err(BridgeError::Panic(msg)) => assert!(msg.contains("panic_at_step 0"), "{msg}"),
            other => panic!("expected a panic, got {other:?}"),
        }
        assert!(matches!(
            with_session(bad, |s| Ok(s.progress())),
            Err(BridgeError::Poisoned(_))
        ));
        assert_eq!(
            with_session(good, |s| Ok(s.advance(Duration::MAX)?)).unwrap(),
            deeptime_core::session::AdvanceStatus::Done
        );
        SESSIONS.remove(bad as u64).unwrap();
        SESSIONS.remove(good as u64).unwrap();
        assert!(matches!(
            with_session(bad, |s| Ok(s.progress())),
            Err(BridgeError::UnknownSession(_))
        ));
    }
}
