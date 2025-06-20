// sleepy_proxy/src/bucket.rs
use std::env;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;

pub struct RateLimiter {
    last_seen: Instant,
    budget: u32,
}
lazy_static! {
    static ref LIMIT_INITIAL: u32 = get_env_u32("LIMIT_INITIAL", 20000);
    static ref LIMIT_IDLE_RATE: u32 = get_env_u32("LIMIT_IDLE_RATE", 200);
    static ref LIMIT_SPEND_RATE: u32 = get_env_u32("LIMIT_SPEND_RATE", 3000);
    static ref LIMIT_CAP: u32 = get_env_u32("LIMIT_CAP", 20000);
    static ref PENALTY_MULTIPLIER: u32 = get_env_u32("PENALTY_MULTIPLIER", 1);
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_seen: Instant::now(),
            budget: *LIMIT_INITIAL,
        }
    }

    pub fn on_response(&mut self, response_time: Duration) {
        let now = Instant::now();
        let idle_ms = now.duration_since(self.last_seen).as_millis() as u32;
        let idle_drain = idle_ms * *LIMIT_IDLE_RATE / 1000;
        let spent = response_time.as_millis() as u32 * *LIMIT_SPEND_RATE / 1000;

        self.budget += spent;
        if idle_drain > self.budget {
            self.budget = 0
        } else {
            self.budget -= idle_drain
        }

        self.last_seen = now;
    }

    pub fn get_budget(&self) -> u32 {
        self.budget
    }

    pub fn get_penalty_delay(&self) -> Duration {
        if self.budget <= *LIMIT_CAP {
            Duration::from_millis(0)
        } else {
            let delay: u64 = ((self.budget - *LIMIT_CAP) * *PENALTY_MULTIPLIER).into();
            println!("DELAY: {}", delay);
            Duration::from_millis(delay)
        }
    }
}

fn get_env_u32(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
