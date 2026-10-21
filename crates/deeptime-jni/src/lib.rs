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

use deeptime_core::compute::ComputeMode;
use deeptime_core::log::Level;
use deeptime_core::session::{PROGRESS_FIELDS, Session, SessionOptions};
use deeptime_core::{CoreError, FORMAT_VERSION};
use deeptime_format::ColumnWorld;
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

/// The CPU level the library was compiled for: `x86-64-v3` for the second x86-64 native, built
/// with `-C target-cpu=x86-64-v3` (decisions log 2026-10-18; plan §6.4), `baseline` otherwise. The
/// loader logs it beside the build it chose, so a v3 file that is not a v3 build shows.
pub const CPU_LEVEL: &str = if cfg!(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi1",
    target_feature = "bmi2",
    target_feature = "f16c",
    target_feature = "fma",
    target_feature = "lzcnt",
    target_feature = "movbe",
)) {
    "x86-64-v3"
} else {
    "baseline"
};

/// `version()`'s string: `<semver>+<git sha>;format=<f>;target=<rust triple>;cpu=<level>`
/// (plan §6.1).
pub fn version_string() -> String {
    format!("{VERSION}+{GIT_SHA};format={FORMAT_VERSION};target={TARGET};cpu={CPU_LEVEL}")
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

/// `createWith` flag: write epoch checkpoints into `<outDir>/epochs/` (docs/design/
/// playability-steering.md §3).
const FLAG_EPOCHS: jint = 1;
/// `createWith` flag: pick up the run already in `outDir` at the preset's stop, from its epoch
/// checkpoints (`Session::resume_epoch`): an epoch picked, a run stopped early or continued.
const FLAG_RESUME: jint = 2;
/// `createWith` flags: the compute mode (`deeptime_core::compute`), `cpu` or `gpu`; neither is
/// `auto` (or `DEEPTIME_COMPUTE` when it is set), both is an error.
const FLAG_COMPUTE_CPU: jint = 4;
const FLAG_COMPUTE_GPU: jint = 8;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_createWith<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    preset_json: JString<'l>,
    content_json: JString<'l>,
    out_dir: JString<'l>,
    threads: jint,
    mem_cap_bytes: jlong,
    flags: jint,
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
        let known = FLAG_EPOCHS | FLAG_RESUME | FLAG_COMPUTE_CPU | FLAG_COMPUTE_GPU;
        if flags & !known != 0 {
            return Err(BridgeError::IllegalArgument(format!(
                "unknown flags {flags:#x} (1 = epochs, 2 = resume, 4 = compute cpu, 8 = compute gpu)"
            )));
        }
        let compute = match (flags & FLAG_COMPUTE_CPU != 0, flags & FLAG_COMPUTE_GPU != 0) {
            (true, true) => {
                return Err(BridgeError::IllegalArgument(
                    "flags 4 (compute cpu) and 8 (compute gpu) exclude each other".into(),
                ));
            }
            (true, false) => Some(ComputeMode::Cpu),
            (false, true) => Some(ComputeMode::Gpu),
            (false, false) => None,
        };
        let options = SessionOptions {
            out_dir: PathBuf::from(out_dir),
            threads: threads as usize,
            mem_cap: mem_cap_bytes as u64,
        };
        let mut session = if flags & FLAG_RESUME != 0 {
            Session::resume_epoch(&preset, &content, options, Arc::new(alloc::AllocGauge))?
        } else {
            Session::create(&preset, &content, options, Arc::new(alloc::AllocGauge))?
        };
        session.set_epochs(flags & FLAG_EPOCHS != 0)?;
        if let Some(m) = compute {
            session.set_compute(m);
        }
        let id = SESSIONS.insert(SessionEntry::new(session));
        GLOBAL_LOG.push(
            Level::Debug,
            format!("session {id} created (flags {flags:#x})"),
        );
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
        with_direct_bytes(env, &dst_rgba, |dst| {
            with_session(session, |s| Ok(s.copy_keyframe(index, layer, dst)?))
        })
    })
}

/// Runs `f` on the memory of the direct buffer `dst_rgba`.
fn with_direct_bytes<R>(
    env: &Env<'_>,
    dst_rgba: &JByteBuffer<'_>,
    f: impl FnOnce(&mut [u8]) -> BridgeResult<R>,
) -> BridgeResult<R> {
    if dst_rgba.is_null() {
        return Err(BridgeError::IllegalArgument("dstRgba is null".into()));
    }
    let not_direct = |_| BridgeError::IllegalArgument("dstRgba is not a direct buffer".into());
    let capacity = env
        .get_direct_buffer_capacity(dst_rgba)
        .map_err(not_direct)?;
    let address = env
        .get_direct_buffer_address(dst_rgba)
        .map_err(not_direct)?;
    if address.is_null() {
        return Err(BridgeError::IllegalArgument(
            "dstRgba is not a direct buffer".into(),
        ));
    }
    // SAFETY: a direct buffer's memory is `capacity` bytes at `address` and stays valid while
    // Java holds `dst_rgba`, which it does for the whole native call `f` runs in; no other Rust
    // reference to it exists.
    let dst = unsafe { std::slice::from_raw_parts_mut(address, capacity) };
    f(dst)
}

/// `watchLive`: asks for the live map of `layer` for the next two seconds (null or "" stops);
/// returns the newest live frame's number (0: none yet). Never waits for a step.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_watchLive<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    layer: JString<'l>,
) -> jlong {
    guard(&mut env, |env| {
        let layer = if layer.is_null() {
            None
        } else {
            Some(layer.try_to_string(env)?)
        };
        with_session(session, |s| Ok(s.watch_live(layer.as_deref()) as jlong))
    })
}

/// `copyLiveFrame`: the newest live frame's RGBA8 into `dstRgba` and its JSON
/// `{"seq","t","stage","layer","w","h"}`; "" before the first. Never waits for a step.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_copyLiveFrame<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    session: jlong,
    dst_rgba: JByteBuffer<'l>,
) -> JString<'l> {
    guard(&mut env, |env| {
        let info = with_direct_bytes(env, &dst_rgba, |dst| {
            with_session(session, |s| Ok(s.copy_live_frame(dst)?))
        })?;
        Ok(env.new_string(info.unwrap_or_default())?)
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

/// The last world-data folder `sampleColumn` opened, with its manifest's modification time: a
/// parity test samples hundreds of columns of one folder, and a folder written again is reopened.
static COLUMN_WORLD: std::sync::Mutex<Option<OpenWorld>> = std::sync::Mutex::new(None);

/// A folder, its manifest's modification time and the evaluator opened on it.
type OpenWorld = (PathBuf, Option<std::time::SystemTime>, Arc<ColumnWorld>);

/// The column evaluator on `dir` (cached, see [`COLUMN_WORLD`]).
pub fn column_world(dir: &std::path::Path) -> Result<Arc<ColumnWorld>, CoreError> {
    let stamp = std::fs::metadata(dir.join(deeptime_format::manifest::MANIFEST))
        .and_then(|m| m.modified())
        .ok();
    let mut cache = COLUMN_WORLD.lock().unwrap_or_else(|p| p.into_inner());
    if let Some((path, at, w)) = cache.as_ref()
        && path == dir
        && *at == stamp
    {
        return Ok(Arc::clone(w));
    }
    let w = Arc::new(ColumnWorld::open(dir)?);
    *cache = Some((dir.to_owned(), stamp, Arc::clone(&w)));
    Ok(w)
}

/// The parity oracle (plan §6.1, §9.2): column (x, z) of a world-data folder as the Rust column
/// evaluator answers it (`deeptime_format::column`), as the JSON `parity/columns.json` holds per
/// column. Not a chunk-generation path: the game reads world data in Java.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_sampleColumn<
    'l,
>(
    mut env: EnvUnowned<'l>,
    _class: JClass<'l>,
    world_data_dir: JString<'l>,
    x: jint,
    z: jint,
) -> JString<'l> {
    guard(&mut env, |env| {
        let dir = PathBuf::from(string_arg(env, &world_data_dir, "worldDataDir")?);
        let w = column_world(&dir)?;
        let json = w.sample(i64::from(x), i64::from(z)).to_json().to_string();
        Ok(env.new_string(json)?)
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
        assert!(rest.starts_with("format=0.6;target="), "{v}");
        assert!(
            rest.ends_with(&format!("target={TARGET};cpu={CPU_LEVEL}")),
            "{v}"
        );
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
