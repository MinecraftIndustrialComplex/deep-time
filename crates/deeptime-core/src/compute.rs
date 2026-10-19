// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where the heavy kernels run (`docs/design/gpu-compute.md`): `auto` uses a hardware GPU when
//! the build has the `gpu` feature and the machine has a usable adapter, `cpu` never does, and
//! `gpu` insists (a software adapter such as lavapipe counts), still falling back to the CPU with
//! a warning when there is none. A kernel that fails on the device (an allocation refused, a
//! validation error, a lost device) reruns its work on the CPU; the CPU path is the reference.

use crate::log::{Level, LogRing};

/// The session's compute mode (run config `compute`, CLI `--compute`, environment
/// [`ComputeMode::ENV`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComputeMode {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

impl ComputeMode {
    /// The environment variable a session's default mode comes from (`auto`, `cpu` or `gpu`).
    pub const ENV: &'static str = "DEEPTIME_COMPUTE";

    pub fn parse(s: &str) -> Option<ComputeMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(ComputeMode::Auto),
            "cpu" => Some(ComputeMode::Cpu),
            "gpu" => Some(ComputeMode::Gpu),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ComputeMode::Auto => "auto",
            ComputeMode::Cpu => "cpu",
            ComputeMode::Gpu => "gpu",
        }
    }

    /// [`ComputeMode::ENV`], or `auto` when it is unset or not a mode.
    pub fn from_env() -> ComputeMode {
        std::env::var(Self::ENV)
            .ok()
            .and_then(|s| ComputeMode::parse(&s))
            .unwrap_or_default()
    }
}

/// The device the GPU kernels use under `mode`, or None for the CPU; logs the choice.
#[cfg(feature = "gpu")]
pub fn device(mode: ComputeMode, log: &LogRing) -> Option<std::sync::Arc<deeptime_gpu::Gpu>> {
    if mode == ComputeMode::Cpu {
        return None;
    }
    match deeptime_gpu::shared(mode == ComputeMode::Gpu) {
        Ok(g) => {
            log.push(
                Level::Info,
                format!("compute: {} on {}", mode.as_str(), g.describe()),
            );
            Some(g)
        }
        Err(e) => {
            let level = if mode == ComputeMode::Gpu {
                Level::Warn
            } else {
                Level::Info
            };
            log.push(
                level,
                format!("compute: {} runs on the CPU ({e})", mode.as_str()),
            );
            None
        }
    }
}

/// Without the `gpu` feature everything runs on the CPU; a forced `gpu` is warned about.
#[cfg(not(feature = "gpu"))]
pub fn warn_without_gpu(mode: ComputeMode, log: &LogRing) {
    if mode == ComputeMode::Gpu {
        log.push(
            Level::Warn,
            "compute: gpu asked for, but this build has no GPU support; running on the CPU",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_parse_and_print() {
        for m in [ComputeMode::Auto, ComputeMode::Cpu, ComputeMode::Gpu] {
            assert_eq!(ComputeMode::parse(m.as_str()), Some(m));
        }
        assert_eq!(ComputeMode::parse(" GPU "), Some(ComputeMode::Gpu));
        assert_eq!(ComputeMode::parse("cuda"), None);
        assert_eq!(ComputeMode::default(), ComputeMode::Auto);
    }
}
