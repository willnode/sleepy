// sleepy_proxy/src/bucket.rs
use std::env;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    last_seen: Instant,
    budget: f64,
}

impl RateLimiter {
    pub fn new() -> Self {
        let cap = get_env_f64("LIMIT_INITIAL", 20000.0);
        Self {
            last_seen: Instant::now(),
            budget: cap,
        }
    }

    pub fn on_response(&mut self, response_time: Duration) {
        let now = Instant::now();
        let idle_ms = now.duration_since(self.last_seen).as_millis() as f64;
        let idle_drain = idle_ms * get_env_f64("LIMIT_IDLE_RATE", 1.0);
        let spent = response_time.as_millis() as f64 * get_env_f64("LIMIT_SPEND_RATE", 3.0);

        self.budget -= spent;
        self.budget += idle_drain;

        let cap = get_env_f64("LIMIT_CAP", 20000.0);
        if self.budget > cap {
            self.budget = cap;
        }

        self.last_seen = now;
    }

    pub fn get_penalty_delay(&self) -> Duration {
        if self.budget >= 0.0 {
            Duration::from_millis(0)
        } else {
            let penalty_factor = get_env_f64("PENALTY_MULTIPLIER", 3.0);
            let delay = (-self.budget * penalty_factor).max(0.0);
            Duration::from_millis(delay as u64)
        }
    }
}

fn get_env_f64(key: &str, default: f64) -> f64 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
