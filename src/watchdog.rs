//! Minimal systemd watchdog client (binary-only module).
//!
//! Implements just enough of the `sd_notify` protocol to send `READY=1`
//! at startup and throttled `WATCHDOG=1` pings from the frame loop, so a
//! wedged process is killed and restarted by systemd (`WatchdogSec=`).
//! Everything no-ops when not running under systemd (`NOTIFY_SOCKET`
//! unset), so desktop and manual runs are unaffected.

use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::net::UnixDatagram;

/// Sends watchdog keep-alives to systemd's notify socket.
pub struct Watchdog {
    #[cfg(unix)]
    socket: Option<UnixDatagram>,
    last_ping: Instant,
}

impl Watchdog {
    /// Connect to `$NOTIFY_SOCKET` if present and announce `READY=1`.
    pub fn new() -> Self {
        let wd = Self {
            #[cfg(unix)]
            socket: connect_notify_socket(),
            last_ping: Instant::now(),
        };
        #[cfg(unix)]
        if wd.socket.is_some() {
            log::info!("systemd watchdog notifications enabled");
        }
        wd.send("READY=1");
        wd
    }

    /// Call once per processed frame; sends `WATCHDOG=1` at most once
    /// per second.
    pub fn ping(&mut self) {
        if self.last_ping.elapsed() >= Duration::from_secs(1) {
            self.send("WATCHDOG=1");
            self.last_ping = Instant::now();
        }
    }

    #[cfg(unix)]
    fn send(&self, msg: &str) {
        if let Some(sock) = &self.socket {
            let _ = sock.send(msg.as_bytes());
        }
    }

    #[cfg(not(unix))]
    fn send(&self, _msg: &str) {}
}

#[cfg(unix)]
fn connect_notify_socket() -> Option<UnixDatagram> {
    let path = std::env::var("NOTIFY_SOCKET").ok()?;
    let sock = UnixDatagram::unbound().ok()?;
    if let Some(name) = path.strip_prefix('@') {
        // Abstract-namespace socket (containers, portable services).
        #[cfg(target_os = "linux")]
        {
            use std::os::linux::net::SocketAddrExt;
            let addr = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).ok()?;
            sock.connect_addr(&addr).ok()?;
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = name;
            return None;
        }
    } else {
        sock.connect(&path).ok()?;
    }
    Some(sock)
}
