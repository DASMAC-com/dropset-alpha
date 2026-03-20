use std::{
    collections::VecDeque,
    time::Duration,
};

use tokio::time::Instant;

/// Tracks a 60-second sliding window of processed requests and uses hysteresis
/// to switch between fast (inline) and slow (batched) mode.
///
/// Thresholds are derived from the configured `min_tx_interval_ms`:
///   - `enter_slow`: 83% of the theoretical max txns/window (~17% headroom)
///   - `exit_slow`: half of `enter_slow` (hysteresis gap)
///   - `drain_interval`: 10x `min_tx_interval` (accumulation period in slow mode)
///
/// For example, with the Solana public RPC defaults (`min_tx_interval_ms = 300`):
///   - max txns/60s = 200, enter_slow = 166, exit_slow = 83, drain_interval = 3s
pub struct RateWindow {
    window: VecDeque<Instant>,
    slow_mode: bool,
    enter_slow: usize,
    exit_slow: usize,
    drain_interval: Duration,
}

/// 60-second sliding window for rate tracking.
const WINDOW_DURATION: Duration = Duration::from_secs(60);

impl RateWindow {
    /// Creates a new `RateWindow` with thresholds derived from `min_tx_interval_ms`.
    ///
    /// All thresholds flow from a single input: the minimum interval between
    /// consecutive `sendTransaction` RPC calls. This value is determined by the
    /// RPC provider's rate limits.
    ///
    /// For the Solana public RPC (devnet/testnet), the binding constraint is:
    ///   **40 requests per 10s per IP for a single RPC method**
    ///   → 4 req/s → 250ms minimum interval.
    ///   We default to 300ms for ~17% headroom.
    ///
    /// ## Derivation
    ///
    /// Given `min_tx_interval_ms`:
    ///
    /// 1. **`max_per_window`** = `60_000 / min_tx_interval_ms`
    ///    The theoretical maximum number of transactions we can submit within
    ///    the 60-second sliding window without exceeding the RPC rate limit.
    ///    e.g. 300ms interval → 60_000 / 300 = 200 txns/min.
    ///
    /// 2. **`enter_slow`** = `max_per_window * 83%`
    ///    We enter slow (batched) mode at 83% of the theoretical max, leaving
    ///    17% headroom so we don't slam into the hard limit. The remaining
    ///    capacity absorbs in-flight requests that were queued before the mode
    ///    switch takes effect.
    ///    e.g. 200 * 0.83 = 166 txns.
    ///
    /// 3. **`exit_slow`** = `enter_slow / 2`
    ///    We only exit slow mode once the window drops to half the entry
    ///    threshold. This hysteresis gap prevents rapid oscillation between
    ///    modes when load hovers near the boundary. The wider the gap, the
    ///    more stable the mode — at the cost of staying in slow mode longer
    ///    than strictly necessary.
    ///    e.g. 166 / 2 = 83 txns.
    ///
    /// 4. **`drain_interval`** = `min_tx_interval_ms * 10`
    ///    In slow mode, we sleep this long before draining the queue. This
    ///    lets multiple requests accumulate so we can batch them into fewer
    ///    transactions. 10x the tx interval means each drain cycle can pack
    ///    up to ~10 transactions' worth of requests (bounded by
    ///    `max_batch_size`), significantly reducing RPC call volume.
    ///    e.g. 300ms * 10 = 3s drain interval.
    pub fn from_interval(min_tx_interval_ms: u64) -> Self {
        let max_per_window = (WINDOW_DURATION.as_millis() as u64) / min_tx_interval_ms.max(1);
        let enter_slow = ((max_per_window * 83) / 100) as usize;
        let exit_slow = enter_slow / 2;
        let drain_interval = Duration::from_millis(min_tx_interval_ms.saturating_mul(10));

        Self {
            window: VecDeque::new(),
            slow_mode: false,
            enter_slow: enter_slow.max(1),
            exit_slow,
            drain_interval,
        }
    }

    pub fn drain_interval(&self) -> Duration {
        self.drain_interval
    }

    pub fn enter_slow_threshold(&self) -> usize {
        self.enter_slow
    }

    pub fn exit_slow_threshold(&self) -> usize {
        self.exit_slow
    }

    pub fn is_slow(&self) -> bool {
        self.slow_mode
    }

    /// Prunes expired entries from the window.
    pub fn prune(&mut self) {
        let now = Instant::now();
        self.window
            .retain(|t| now.duration_since(*t) < WINDOW_DURATION);
    }

    /// Records `count` new requests in the window and re-evaluates the mode.
    /// Returns the new mode (true = slow).
    pub fn record(&mut self, count: usize) -> bool {
        let now = Instant::now();
        for _ in 0..count {
            self.window.push_back(now);
        }

        if self.slow_mode {
            if self.window.len() < self.exit_slow {
                self.slow_mode = false;
            }
        } else if self.window.len() > self.enter_slow {
            self.slow_mode = true;
        }

        self.slow_mode
    }

    /// Attempts to exit slow mode if the window has drained enough.
    /// Should be called after the drain interval sleep.
    pub fn try_exit_slow(&mut self) {
        self.prune();
        if self.window.len() < self.exit_slow {
            self.slow_mode = false;
        }
    }

    pub fn window_len(&self) -> usize {
        self.window.len()
    }
}

#[cfg(test)]
mod tests {
    use tokio::time;

    use super::*;

    /// Creates a RateWindow with Solana public RPC defaults (300ms interval).
    fn default_rate() -> RateWindow {
        RateWindow::from_interval(300)
    }

    #[test]
    fn thresholds_derived_from_interval() {
        let rate = RateWindow::from_interval(300);
        // 60_000ms / 300ms = 200 max. 200 * 0.83 = 166. 166 / 2 = 83.
        assert_eq!(rate.enter_slow_threshold(), 166);
        assert_eq!(rate.exit_slow_threshold(), 83);
        assert_eq!(rate.drain_interval(), Duration::from_millis(3000));
    }

    #[test]
    fn faster_rpc_raises_thresholds() {
        let rate = RateWindow::from_interval(100);
        // 60_000ms / 100ms = 600 max. 600 * 0.83 = 498. 498 / 2 = 249.
        assert_eq!(rate.enter_slow_threshold(), 498);
        assert_eq!(rate.exit_slow_threshold(), 249);
        assert_eq!(rate.drain_interval(), Duration::from_millis(1000));
    }

    #[tokio::test(start_paused = true)]
    async fn stays_fast_under_low_load() {
        let mut rate = default_rate();

        rate.record(rate.exit_slow_threshold() - 1);

        assert!(!rate.is_slow());
    }

    #[tokio::test(start_paused = true)]
    async fn enters_slow_above_threshold() {
        let mut rate = default_rate();

        rate.record(rate.enter_slow_threshold() + 1);

        assert!(rate.is_slow());
    }

    #[tokio::test(start_paused = true)]
    async fn hysteresis_prevents_thrashing() {
        let mut rate = default_rate();

        // Exceed enter threshold at t=0 → enters slow mode.
        rate.record(rate.enter_slow_threshold() + 1);
        assert!(rate.is_slow());

        // 30s later: all entries are still within the 60s window.
        // Count > exit threshold, so we stay in slow mode.
        time::advance(Duration::from_secs(30)).await;
        rate.try_exit_slow();
        assert!(rate.is_slow());

        // 61s later: every entry has expired out of the window.
        // 0 < exit threshold → exits slow mode.
        time::advance(Duration::from_secs(31)).await;
        rate.try_exit_slow();
        assert_eq!(rate.window_len(), 0);
        assert!(!rate.is_slow());
    }

    #[tokio::test(start_paused = true)]
    async fn exits_slow_below_exit_threshold() {
        let mut rate = default_rate();

        rate.record(rate.enter_slow_threshold() + 1);
        assert!(rate.is_slow());

        // Wait for the entire window to expire, then record below exit threshold.
        time::advance(Duration::from_secs(61)).await;
        rate.prune();
        rate.record(rate.exit_slow_threshold() - 1);
        assert!(!rate.is_slow());
    }

    #[tokio::test(start_paused = true)]
    async fn does_not_exit_slow_between_thresholds() {
        let mut rate = default_rate();
        let between = (rate.exit_slow_threshold() + rate.enter_slow_threshold()) / 2;

        rate.record(rate.enter_slow_threshold() + 1);
        assert!(rate.is_slow());

        // Window expires, then a count between thresholds arrives.
        // Must drop *below* exit threshold to leave slow mode.
        time::advance(Duration::from_secs(61)).await;
        rate.prune();
        rate.record(between);
        assert!(rate.is_slow());
    }

    #[tokio::test(start_paused = true)]
    async fn window_prunes_expired_entries() {
        let mut rate = default_rate();
        let count = rate.exit_slow_threshold() - 1;

        rate.record(count);
        assert_eq!(rate.window_len(), count);

        // After 61s every entry is older than the 60s window → all pruned.
        time::advance(Duration::from_secs(61)).await;
        rate.prune();
        assert_eq!(rate.window_len(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn rapid_burst_then_cooldown_cycle() {
        let mut rate = default_rate();

        // Burst above enter threshold → enters slow mode.
        rate.record(rate.enter_slow_threshold() + 1);
        assert!(rate.is_slow());

        // No new requests arrive. The processor drains every drain_interval.
        // After enough intervals (>60s total), the window fully expires
        // and we drop below exit threshold → exits slow mode.
        for _ in 0..25 {
            time::advance(rate.drain_interval()).await;
            rate.try_exit_slow();
            if !rate.is_slow() {
                break;
            }
        }
        assert!(!rate.is_slow(), "Should have exited slow mode after window expired");

        // Normal low traffic resumes → stays fast.
        rate.record(10);
        assert!(!rate.is_slow());
    }
}
