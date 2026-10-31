//! Request budgets are shared by every branch of a composite tool invocation.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use crate::client::{Error, Result};

#[derive(Clone)]
pub(crate) struct ClientPolicy {
    pub max_requests: u32,
    pub max_retries: u32,
    pub tool_timeout: Duration,
}

impl Default for ClientPolicy {
    fn default() -> Self {
        Self {
            max_requests: 200,
            max_retries: 2,
            tool_timeout: Duration::from_secs(120),
        }
    }
}

pub(crate) struct CallBudget(AtomicU32);

impl CallBudget {
    pub fn new(limit: u32) -> Self {
        Self(AtomicU32::new(limit))
    }

    pub fn take(&self) -> Result<()> {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                remaining.checked_sub(1)
            })
            .map(|_| ())
            .map_err(|_| Error::Budget)
    }
}

pub(crate) struct RateLimit {
    per_minute: u32,
    state: Mutex<(Instant, f64)>,
}

impl RateLimit {
    pub fn new(per_minute: u32) -> Self {
        Self {
            per_minute,
            state: Mutex::new((Instant::now(), f64::from(per_minute))),
        }
    }

    pub fn check(&self) -> Option<u64> {
        self.check_at(Instant::now())
    }

    fn check_at(&self, now: Instant) -> Option<u64> {
        if self.per_minute == 0 {
            return None;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        // Concurrent callers can acquire the mutex in a different order from their timestamps.
        let now = now.max(state.0);
        let rate = f64::from(self.per_minute) / 60.;
        state.1 = (state.1 + now.duration_since(state.0).as_secs_f64() * rate)
            .min(f64::from(self.per_minute));
        state.0 = now;
        if state.1 >= 1. {
            state.1 -= 1.;
            None
        } else {
            Some(((1. - state.1) / rate).ceil() as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_bucket_refills_and_disabled_limits_accept_requests() {
        let limit = RateLimit::new(2);
        let now = Instant::now();
        assert_eq!(limit.check_at(now), None);
        assert_eq!(limit.check_at(now), None);
        assert_eq!(limit.check_at(now), Some(30));
        assert_eq!(limit.check_at(now + Duration::from_secs(30)), None);
        assert_eq!(limit.check_at(now + Duration::from_secs(30)), Some(30));
        assert_eq!(RateLimit::new(0).check(), None);
    }
}
