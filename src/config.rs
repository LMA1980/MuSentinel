//! Runtime configuration and threshold parameters for `MuSentinel`.

use std::path::PathBuf;
use std::time::Duration;

/// Default local `MuObserve-collector` HTTP base URL.
pub const DEFAULT_COLLECTOR_URL: &str = "http://127.0.0.1:4318";

/// Default polling interval in seconds (`5s`).
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;

/// Default block I/O `await` warning threshold in milliseconds (`500 ms`).
pub const DEFAULT_DISK_AWAIT_WARN_MS: f64 = 500.0;

/// Default runaway process elapsed threshold in seconds (`900s` / 15 minutes).
pub const DEFAULT_RUNAWAY_SECS: u64 = 900;

/// Default PSI `avg10` warning threshold percentage (`10.0%`).
pub const DEFAULT_PSI_WARN_PCT: f64 = 10.0;

/// Runtime configuration for `MuSentinel` loaded from environment variables.
#[derive(Debug, Clone)]
pub struct SentinelConfig {
    /// Node identifier reported in `OtelMetricEntry` and `OtelLogEntry` (`MUOBSERVE_NODE_ID`).
    pub node_id: String,
    /// Host CPU architecture (`x86_64` or `riscv64gc`).
    pub arch: String,
    /// Base URL for `MuObserve-collector` (`MUOBSERVE_COLLECTOR_URL`).
    pub collector_url: String,
    /// Polling interval between procfs/sysfs sweeps (`MUSENTINEL_POLL_SECS`).
    pub poll_interval: Duration,
    /// Disk read/write `await` threshold in milliseconds before emitting a warning log (`MUSENTINEL_DISK_AWAIT_WARN_MS`).
    pub disk_await_warn_ms: f64,
    /// PSI `avg10` stall percentage threshold before emitting a warning log (`MUSENTINEL_PSI_WARN_PCT`).
    pub psi_warn_pct: f64,
    /// Maximum runtime in seconds for monitored build/test processes before alerting (`MUSENTINEL_RUNAWAY_SECS`).
    pub runaway_process_secs: u64,
    /// Root path for procfs (defaults to `/proc`).
    pub proc_root: PathBuf,
    /// Root path for sysfs (defaults to `/sys`).
    pub sys_root: PathBuf,
    /// Path to `/dev/kmsg` (defaults to `/dev/kmsg`).
    pub kmsg_path: PathBuf,
    /// Whether to enforce Landlock V3+ rules at startup (`MUSENTINEL_ENFORCE_LANDLOCK`).
    pub enforce_landlock_on_linux: bool,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        let host_arch = std::env::consts::ARCH;
        let arch = if matches!(host_arch, "x86_64" | "riscv64" | "riscv64gc") {
            host_arch.to_string()
        } else {
            "riscv64gc".to_string()
        };
        Self {
            node_id: "worker-node".to_string(),
            arch,
            collector_url: DEFAULT_COLLECTOR_URL.to_string(),
            poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
            disk_await_warn_ms: DEFAULT_DISK_AWAIT_WARN_MS,
            psi_warn_pct: DEFAULT_PSI_WARN_PCT,
            runaway_process_secs: DEFAULT_RUNAWAY_SECS,
            proc_root: PathBuf::from("/proc"),
            sys_root: PathBuf::from("/sys"),
            kmsg_path: PathBuf::from("/dev/kmsg"),
            enforce_landlock_on_linux: false,
        }
    }
}

impl SentinelConfig {
    /// Constructs [`SentinelConfig`] from environment variables with safe defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(node_id) = std::env::var("MUOBSERVE_NODE_ID") {
            if !node_id.trim().is_empty() {
                cfg.node_id = node_id.trim().to_string();
            }
        }
        if let Ok(url) = std::env::var("MUOBSERVE_COLLECTOR_URL") {
            if !url.trim().is_empty() {
                cfg.collector_url = url.trim().trim_end_matches('/').to_string();
            }
        }
        if let Ok(secs) = std::env::var("MUSENTINEL_POLL_SECS") {
            if let Ok(parsed) = secs.trim().parse::<u64>() {
                if parsed > 0 {
                    cfg.poll_interval = Duration::from_secs(parsed);
                }
            }
        }
        if let Ok(ms) = std::env::var("MUSENTINEL_DISK_AWAIT_WARN_MS") {
            if let Ok(parsed) = ms.trim().parse::<f64>() {
                if parsed > 0.0 {
                    cfg.disk_await_warn_ms = parsed;
                }
            }
        }
        if let Ok(pct) = std::env::var("MUSENTINEL_PSI_WARN_PCT") {
            if let Ok(parsed) = pct.trim().parse::<f64>() {
                if parsed > 0.0 {
                    cfg.psi_warn_pct = parsed;
                }
            }
        }
        if let Ok(secs) = std::env::var("MUSENTINEL_RUNAWAY_SECS") {
            if let Ok(parsed) = secs.trim().parse::<u64>() {
                if parsed > 0 {
                    cfg.runaway_process_secs = parsed;
                }
            }
        }
        if let Ok(enforce) = std::env::var("MUSENTINEL_ENFORCE_LANDLOCK") {
            cfg.enforce_landlock_on_linux =
                enforce.trim() == "1" || enforce.trim().eq_ignore_ascii_case("true");
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_invariants() {
        let cfg = SentinelConfig::default();
        assert_eq!(cfg.collector_url, "http://127.0.0.1:4318");
        assert_eq!(cfg.poll_interval, Duration::from_secs(5));
        assert!((cfg.disk_await_warn_ms - 500.0).abs() < f64::EPSILON);
        assert_eq!(cfg.runaway_process_secs, 900);
    }
}
