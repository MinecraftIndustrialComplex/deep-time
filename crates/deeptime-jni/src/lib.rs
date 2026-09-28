// SPDX-License-Identifier: LGPL-3.0-or-later
//! The JNI bridge (plan §6): marshalling, the session handle table and panic guards. The only
//! crate that depends on `jni`.

pub use deeptime_core::VERSION;

#[cfg(test)]
mod tests {
    #[test]
    fn links_core() {
        assert!(!super::VERSION.is_empty());
    }
}
