//! Auto-tuner for semantic indexing concurrency.
//!
//! Resolves how many rayon threads to use for AST parsing and how many concurrent
//! embedding requests to issue against the local model. Distinguishes between
//! CPU-bound work (parse), which scales with logical cores, and network-bound
//! work (embed), which is capped to avoid overwhelming the local ONNX server
//! (see commit `90da256` — embedding callsites crashed at 16+ concurrent).
//!
//! Precedence: CLI override (`-j`) > `[semantic].concurrency` config > auto.
//! Auto uses [`std::thread::available_parallelism`] so cgroup/affinity limits
//! are respected under WSL, Docker, and CI containers.

use std::num::NonZeroUsize;

/// Resolved thread counts for semantic indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedConcurrency {
    /// Threads for CPU-bound AST parsing (rayon pool size).
    pub parse_threads: NonZeroUsize,
    /// Max concurrent embedding requests in flight.
    pub embed_threads: NonZeroUsize,
}

/// Options controlling the auto-tuner.
#[derive(Debug, Clone, Copy)]
pub struct ResolveOptions {
    /// Upper bound on concurrent embed requests. Defaults to 4.
    pub embed_cap: NonZeroUsize,
    /// Override for the "auto" branch. `None` reads
    /// [`std::thread::available_parallelism`].
    pub available_parallelism: Option<NonZeroUsize>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            embed_cap: NonZeroUsize::new(DEFAULT_EMBED_CAP)
                .expect("DEFAULT_EMBED_CAP must be non-zero"),
            available_parallelism: None,
        }
    }
}

/// Default cap on concurrent embed requests. Picked to stay below the local
/// ONNX server's crash threshold (see commit `90da256`).
pub const DEFAULT_EMBED_CAP: usize = 4;

/// Counting semaphore guarding concurrent embed calls. Backed by a
/// `parking_lot::Mutex<Vec<()>>` to avoid pulling in a separate dependency
/// when one is already in the workspace (`parking_lot` at `Cargo.toml:37`).
///
/// Each call to [`acquire`] blocks until a permit is available, then returns
/// a [`EmbedPermit`] that releases the slot on drop.
#[derive(Debug)]
pub struct EmbedSemaphore {
    inner: parking_lot::Mutex<usize>,
    cond: parking_lot::Condvar,
    permits: usize,
}

impl EmbedSemaphore {
    pub fn new(permits: usize) -> Self {
        let permits = permits.max(1);
        Self {
            inner: parking_lot::Mutex::new(permits),
            cond: parking_lot::Condvar::new(),
            permits,
        }
    }

    /// Block until a permit is available, returning a guard.
    pub fn acquire(&self) -> EmbedPermit<'_> {
        let mut guard = self.inner.lock();
        while *guard == 0 {
            self.cond.wait(&mut guard);
        }
        *guard -= 1;
        EmbedPermit { semaphore: self }
    }

    #[cfg(test)]
    fn available(&self) -> usize {
        *self.inner.lock()
    }

    #[cfg(test)]
    fn total(&self) -> usize {
        self.permits
    }
}

/// RAII guard for an embed slot. Releases on drop.
pub struct EmbedPermit<'a> {
    semaphore: &'a EmbedSemaphore,
}

impl Drop for EmbedPermit<'_> {
    fn drop(&mut self) {
        let mut guard = self.semaphore.inner.lock();
        *guard += 1;
        if *guard <= self.semaphore.permits {
            self.semaphore.cond.notify_one();
        }
    }
}

/// Resolve parse/embed thread counts given an optional CLI override, an
/// optional config value, and tuning options.
///
/// Precedence:
/// 1. `cli_override` (when `Some`) wins outright.
/// 2. `config_value` (when `Some` and `cli_override` is `None`) wins.
/// 3. Auto = `min(available_parallelism, embed_cap)` for both, clamped ≥ 1.
pub fn resolve_semantic_concurrency(
    cli_override: Option<usize>,
    config_value: Option<usize>,
    opts: ResolveOptions,
) -> ResolvedConcurrency {
    let raw = cli_override
        .or(config_value)
        .unwrap_or_else(|| opts.available_parallelism.map_or(1, |n| n.get()));

    let raw_nz = NonZeroUsize::new(raw).unwrap_or(NonZeroUsize::new(1).expect("1 is non-zero"));

    let parse_threads = raw_nz;
    let embed_threads = std::cmp::min(raw_nz, opts.embed_cap);

    ResolvedConcurrency {
        parse_threads,
        embed_threads,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nz(n: usize) -> NonZeroUsize {
        NonZeroUsize::new(n).expect("test value must be non-zero")
    }

    #[test]
    fn cli_override_wins_over_config() {
        let opts = ResolveOptions {
            available_parallelism: Some(nz(32)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(Some(2), Some(16), opts);
        assert_eq!(r.parse_threads.get(), 2);
        assert_eq!(r.embed_threads.get(), 2);
    }

    #[test]
    fn config_value_wins_when_no_cli() {
        let opts = ResolveOptions {
            available_parallelism: Some(nz(32)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(None, Some(6), opts);
        assert_eq!(r.parse_threads.get(), 6);
        // embed cap is 4 by default
        assert_eq!(r.embed_threads.get(), 4);
    }

    #[test]
    fn auto_default_uses_available_parallelism() {
        let opts = ResolveOptions {
            available_parallelism: Some(nz(16)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(None, None, opts);
        assert_eq!(r.parse_threads.get(), 16);
        assert_eq!(r.embed_threads.get(), 4);
    }

    #[test]
    fn auto_falls_back_to_one_when_parallelism_unknown() {
        // available_parallelism returns Err on some sandboxed envs;
        // the caller passes None to indicate "couldn't detect".
        let opts = ResolveOptions {
            available_parallelism: None,
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(None, None, opts);
        assert_eq!(r.parse_threads.get(), 1);
        assert_eq!(r.embed_threads.get(), 1);
    }

    #[test]
    fn embed_threads_capped_independently() {
        // Large parse count must not bleed into embed count.
        let opts = ResolveOptions {
            available_parallelism: Some(nz(64)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(Some(20), None, opts);
        assert_eq!(r.parse_threads.get(), 20);
        assert_eq!(r.embed_threads.get(), 4);
    }

    #[test]
    fn embed_threads_match_parse_when_below_cap() {
        let opts = ResolveOptions {
            available_parallelism: Some(nz(64)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(Some(2), None, opts);
        assert_eq!(r.parse_threads.get(), 2);
        assert_eq!(r.embed_threads.get(), 2);
    }

    #[test]
    fn custom_embed_cap_is_respected() {
        let opts = ResolveOptions {
            available_parallelism: Some(nz(32)),
            embed_cap: nz(2),
        };
        let r = resolve_semantic_concurrency(None, Some(8), opts);
        assert_eq!(r.parse_threads.get(), 8);
        assert_eq!(r.embed_threads.get(), 2);
    }

    #[test]
    fn zero_in_config_is_clamped_to_one() {
        // We treat Some(0) as "let auto decide" rather than failing —
        // this is forgiving for hand-edited configs and matches the
        // `> 0` validation's job to catch it at config load time.
        let opts = ResolveOptions {
            available_parallelism: Some(nz(8)),
            ..Default::default()
        };
        let r = resolve_semantic_concurrency(None, Some(0), opts);
        // Should not panic; result must be ≥ 1.
        assert!(r.parse_threads.get() >= 1);
        assert!(r.embed_threads.get() >= 1);
    }

    #[test]
    fn semaphore_releases_on_drop() {
        let sem = EmbedSemaphore::new(2);
        assert_eq!(sem.available(), 2);
        let p1 = sem.acquire();
        assert_eq!(sem.available(), 1);
        let p2 = sem.acquire();
        assert_eq!(sem.available(), 0);
        drop(p1);
        assert_eq!(sem.available(), 1);
        drop(p2);
        assert_eq!(sem.available(), 2);
    }

    #[test]
    fn semaphore_zero_cap_clamps_to_one() {
        // The cap is the floor, not the ceiling — even 0 should still
        // permit one concurrent call to keep the system responsive.
        let sem = EmbedSemaphore::new(0);
        assert_eq!(sem.total(), 1);
        let _p = sem.acquire();
        assert_eq!(sem.available(), 0);
    }
}
