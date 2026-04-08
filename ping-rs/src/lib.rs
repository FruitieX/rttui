//! Vendored fork of [ping-rs](https://github.com/ruxo/ping-rs) with source
//! address binding support. Original license: MIT.

#[cfg(windows)]
mod windows_ping;
#[cfg(unix)]
mod linux_ping;

use std::io;
use std::net::IpAddr;
use std::time::Duration;

/// Contains constant values represent general errors.
#[allow(non_snake_case)]
pub mod IpStatus {
    #![allow(non_upper_case_globals)]

    pub type Type = u32;
    pub const Success: Type = 0;
    pub const DestinationNetworkUnreachable: Type = 11002;
    pub const DestinationHostUnreachable: Type = 11003;
    pub const DestinationProtocolUnreachable: Type = 11004;
    pub const DestinationPortUnreachable: Type = 11005;
    pub const DestinationProhibited: Type = 11019;
    pub const NoResources: Type = 11006;
    pub const BadOption: Type = 11007;
    pub const HardwareError: Type = 11008;
    pub const PacketTooBig: Type = 11009;
    pub const TimedOut: Type = 11010;
    pub const BadRoute: Type = 11012;
    pub const TtlExpired: Type = 11013;
    pub const TtlReassemblyTimeExceeded: Type = 11014;
    pub const ParameterProblem: Type = 11015;
    pub const SourceQuench: Type = 11016;
    pub const BadDestination: Type = 11018;
    pub const DestinationUnreachable: Type = 11040;
    pub const TimeExceeded: Type = 11041;
    pub const BadHeader: Type = 11042;
    pub const UnrecognizedNextHeader: Type = 11043;
    pub const IcmpError: Type = 11044;
    pub const DestinationScopeMismatch: Type = 11045;
    pub const GeneralFailure: Type = 11050;
}

#[derive(Debug, Clone)]
pub struct PingOptions {
    /// Package TTL
    pub ttl: u8,
    /// Socket's Dont Fragment
    pub dont_fragment: bool,
    /// Source IP address to bind to (for interface selection)
    pub source_addr: Option<IpAddr>,
}

/// Ping reply contains the destination address (from ICMP reply) and Round-Trip Time
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PingReply {
    /// Destination address from ICMP reply
    pub address: IpAddr,
    /// Round-Trip Time in milliseconds
    pub rtt: u32,
}

/// Ping errors
#[derive(Debug, Clone)]
pub enum PingError {
    /// Bad request parameters
    BadParameter(&'static str),
    /// Unspecific OS errors
    OsError(u32, String),
    /// General Ping errors
    IpError(IpStatus::Type),
    /// Ping timed out
    TimedOut,
    /// I/O async pending
    IoPending,
    /// size of data buffer for ping is too big
    DataSizeTooBig(usize),
}

impl From<io::Error> for PingError {
    fn from(value: io::Error) -> Self {
        if value.kind() == io::ErrorKind::WouldBlock {
            PingError::IoPending
        } else {
            PingError::OsError(value.raw_os_error().unwrap_or(-1) as u32, value.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, PingError>;
pub type PingApiOutput = Result<PingReply>;

#[cfg(windows)]
use windows_ping as ping_mod;

#[cfg(unix)]
use linux_ping as ping_mod;

/// Send ICMP Echo package (ping) to the given address.
#[inline(always)]
pub fn send_ping(addr: &IpAddr, timeout: Duration, data: &[u8], options: Option<&PingOptions>) -> PingApiOutput {
    ping_mod::send_ping(addr, timeout, data, options)
}
