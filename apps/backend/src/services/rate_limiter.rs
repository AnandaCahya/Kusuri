use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct PairingRateLimiter {
    // Map of IP -> (failed_attempts, last_attempt_time)
    failures: RwLock<HashMap<String, (u32, Instant)>>,
}

impl PairingRateLimiter {
    pub fn new() -> Self {
        Self {
            failures: RwLock::new(HashMap::new()),
        }
    }

    /// Check if the given client identifier (e.g. IP) is allowed to attempt pairing
    pub async fn check_allowed(&self, key: &str) -> bool {
        let mut map = self.failures.write().await;
        // Expire entries older than 5 minutes (300 seconds)
        map.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(300));

        if let Some((count, time)) = map.get(key) {
            if *count >= 3 && time.elapsed() < Duration::from_secs(300) {
                return false;
            }
        }
        true
    }

    /// Record a failed pairing attempt. Returns the new failure count.
    pub async fn record_failure(&self, key: &str) -> u32 {
        let mut map = self.failures.write().await;
        let entry = map.entry(key.to_string()).or_insert((0, Instant::now()));
        entry.0 += 1;
        entry.1 = Instant::now();
        entry.0
    }

    /// Reset failure count upon a successful pairing
    pub async fn reset(&self, key: &str) {
        let mut map = self.failures.write().await;
        map.remove(key);
    }
}
