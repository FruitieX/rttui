#![cfg(unix)]

mod v4;
mod v6;
mod icmp_header;

use std::io::Write;
use std::mem;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use crate::{IpStatus, PingError, PingOptions, PingReply, Result};
use crate::linux_ping::icmp_header::{ICMP_HEADER_SIZE, IcmpEchoHeader};

const TOKEN_SIZE: usize = 24;

pub fn send_ping(addr: &IpAddr, timeout: Duration, data: &[u8], options: Option<&PingOptions>) -> Result<PingReply> {
    let mut context = match addr {
        IpAddr::V4(_) => PingContext::new::<Ipv4Addr>(addr, timeout, data, options)?,
        IpAddr::V6(_) => PingContext::new::<Ipv6Addr>(addr, timeout, data, options)?,
    };
    context.ping()?;
    match (context.wait_reply)(&context.socket, context.start_ts) {
        Err(PingError::IoPending) => Err(PingError::TimedOut),
        v => v,
    }
}

// INTERNAL

fn validate_timeout(timeout: Duration) -> Result<Duration> {
    if timeout.le(&Duration::ZERO) {
        Err(PingError::BadParameter("timeout"))
    } else {
        Ok(timeout)
    }
}

pub(crate) struct PingContext {
    ident: u16,
    sequence: u16,
    destination: SocketAddr,
    payload: Vec<u8>,
    socket: Socket,
    #[allow(dead_code)]
    timeout: Duration,
    start_ts: Instant,
    wait_reply: fn(&Socket, Instant) -> Result<PingReply>,
}

const MTU: usize = 1500;

impl PingContext {
    fn new<P: Proto>(
        addr: &IpAddr,
        timeout: Duration,
        data: &[u8],
        options: Option<&PingOptions>,
    ) -> Result<PingContext> {
        let timeout = validate_timeout(timeout)?;
        let payload = make_data::<P>(data)?;

        let socket = create_socket::<P>()?;

        // Bind to source address if specified (for interface selection)
        if let Some(source) = options.and_then(|o| o.source_addr) {
            let bind_addr = SocketAddr::new(source, 0);
            socket.bind(&SockAddr::from(bind_addr))?;
        }

        if let Some(v) = options.map(|o| o.ttl) {
            socket.set_ttl(v as u32)?;
        }
        socket.set_read_timeout(Some(timeout))?;

        let destination = SocketAddr::new(*addr, 0);
        let process_id = std::process::id() as u16;

        Ok(PingContext {
            ident: process_id,
            sequence: 0,
            destination,
            payload,
            socket,
            timeout,
            start_ts: Instant::now(),
            wait_reply: wait_reply::<P>,
        })
    }

    fn ping(&mut self) -> Result<()> {
        self.sequence += 1;
        set_request_data(&mut self.payload, self.ident, self.sequence);

        let addr: SockAddr = self.destination.into();
        self.start_ts = Instant::now();
        let sent = self.socket.send_to(&self.payload, &addr)?;
        assert_eq!(sent, self.payload.len());
        Ok(())
    }
}

fn wait_reply<P: Proto>(socket: &Socket, start_ts: Instant) -> Result<PingReply> {
    let mut buffer: [MaybeUninit<u8>; MTU] = unsafe { MaybeUninit::uninit().assume_init() };
    let (size, addr) = socket.recv_from(&mut buffer)?;
    debug_assert_ne!(size, 0);
    let reply_buffer = unsafe { mem::transmute::<_, [u8; MTU]>(buffer) };

    let header = IcmpEchoHeader::get_ref(&reply_buffer);
    if header.r#type != P::ECHO_REPLY_TYPE || header.code != P::ECHO_REPLY_CODE {
        return Err(PingError::IpError(IpStatus::BadHeader));
    }

    Ok(PingReply {
        address: addr.as_socket().unwrap().ip(),
        rtt: (start_ts.elapsed().as_secs_f64() * 1000.) as u32,
    })
}

struct SocketConfig(Domain, Protocol);

// idea from tokio-ping
trait Proto {
    const ECHO_REQUEST_TYPE: u8;
    const ECHO_REQUEST_CODE: u8;
    const ECHO_REPLY_TYPE: u8;
    const ECHO_REPLY_CODE: u8;
    const SOCKET_CONFIG: SocketConfig;

    fn get_reply_header(reply: &[u8]) -> Result<&IcmpEchoHeader>;
}

fn create_socket<P: Proto>() -> Result<Socket> {
    let SocketConfig(domain, protocol) = P::SOCKET_CONFIG;
    Socket::new_raw(domain, Type::DGRAM, Some(protocol)).map_err(|x| x.into())
}

fn make_data<P: Proto>(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() > TOKEN_SIZE {
        return Err(PingError::DataSizeTooBig(TOKEN_SIZE));
    }

    let mut buffer = vec![0; ICMP_HEADER_SIZE + data.len()];
    let payload = &mut buffer[ICMP_HEADER_SIZE..];
    if payload.write(data).is_err() {
        return Err(PingError::BadParameter("data"));
    }
    let header = IcmpEchoHeader::get_mut_ref(&mut buffer);

    header.r#type = P::ECHO_REQUEST_TYPE;
    header.code = P::ECHO_REQUEST_CODE;
    write_checksum(&mut buffer);

    Ok(buffer)
}

fn set_request_data(data: &mut [u8], ident: u16, sequence: u16) {
    let header = IcmpEchoHeader::get_mut_ref(data);
    header.set_ident(ident);
    header.set_seq(sequence);
    write_checksum(data);
}

fn write_checksum(buffer: &mut [u8]) {
    let mut sum = 0u32;
    for word in buffer.chunks(2) {
        let mut part = u16::from(word[0]) << 8;
        if word.len() > 1 {
            part += u16::from(word[1]);
        }
        sum = sum.wrapping_add(u32::from(part));
    }

    while (sum >> 16) > 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    let sum = !sum as u16;

    IcmpEchoHeader::get_mut_ref(buffer).set_checksum(sum);
}
