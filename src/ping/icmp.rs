use super::{PingResult, Pinger};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

/// ICMP ping implementation using ping_rs
pub struct IcmpPinger {
    target: IpAddr,
    interval_ms: u64,
    timeout_ms: u64,
}

impl IcmpPinger {
    pub fn new(target: IpAddr, interval_ms: u64, timeout_ms: u64) -> Self {
        Self {
            target,
            interval_ms,
            timeout_ms,
        }
    }
}

impl Pinger for IcmpPinger {
    fn start(
        self: Box<Self>,
        tx: mpsc::UnboundedSender<PingResult>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut seq: u64 = 0;
            let mut ticker = interval(Duration::from_millis(self.interval_ms));
            let prev_rtt: std::sync::Arc<std::sync::Mutex<Option<Duration>>> =
                std::sync::Arc::new(std::sync::Mutex::new(None));

            loop {
                ticker.tick().await;

                let sent_at = Instant::now();
                seq += 1;
                let current_seq = seq;
                let target = self.target;
                let timeout = Duration::from_millis(self.timeout_ms);
                let tx_clone = tx.clone();
                let prev_rtt_clone = prev_rtt.clone();

                // Spawn ping in background so we don't block the interval
                tokio::spawn(async move {
                    let ping_start = Instant::now();
                    let result = tokio::task::spawn_blocking(move || {
                        ping_rs::send_ping(&target, timeout, &[1, 2, 3, 4], None)
                    })
                    .await;

                    let ping_result = match result {
                        Ok(Ok(_reply)) => {
                            // Measure RTT ourselves for sub-millisecond precision
                            // (ping_rs on Windows only returns whole milliseconds)
                            let rtt = ping_start.elapsed();
                            let prev = {
                                let mut guard = prev_rtt_clone.lock().unwrap();
                                let prev = *guard;
                                *guard = Some(rtt);
                                prev
                            };
                            PingResult::success(current_seq, rtt, sent_at, prev)
                        }
                        _ => {
                            // Clear previous RTT on timeout
                            *prev_rtt_clone.lock().unwrap() = None;
                            PingResult::timeout(current_seq, sent_at)
                        }
                    };

                    let _ = tx_clone.send(ping_result);
                });
            }
        })
    }
}
