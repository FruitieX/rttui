use std::ffi::c_void;
use std::net::{IpAddr, Ipv4Addr};

use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::IpHelper::{
    ICMP_ECHO_REPLY, IcmpHandle, IcmpSendEcho2, IP_OPTION_INFORMATION,
};

use crate::windows_ping::{IcmpEcho, PingRawReply};

// IcmpSendEcho2Ex supports source address binding for IPv4 ICMP.
// Declared via raw FFI since it may not be available in all windows crate versions.
// The function is exported by iphlpapi.dll (already linked by the windows crate features).
#[link(name = "iphlpapi")]
extern "system" {
    fn IcmpSendEcho2Ex(
        IcmpHandle: isize,
        Event: isize,
        ApcRoutine: *const c_void,
        ApcContext: *const c_void,
        SourceAddress: u32,
        DestinationAddress: u32,
        RequestData: *const c_void,
        RequestSize: u16,
        RequestOptions: *const IP_OPTION_INFORMATION,
        ReplyBuffer: *mut c_void,
        ReplySize: u32,
        Timeout: u32,
    ) -> u32;
}

impl IcmpEcho for Ipv4Addr {
    fn send(
        &self,
        handle: IcmpHandle,
        event: Option<HANDLE>,
        data: *const c_void,
        data_len: u16,
        options: *const IP_OPTION_INFORMATION,
        reply_buffer: *mut c_void,
        reply_buffer_len: u32,
        timeout: u32,
        source_addr: Option<IpAddr>,
    ) -> u32 {
        unsafe {
            let destination_address = *((&self.octets() as *const u8) as *const u32);

            if let Some(IpAddr::V4(src)) = source_addr {
                let source = *((&src.octets() as *const u8) as *const u32);
                IcmpSendEcho2Ex(
                    handle.0,
                    event.map_or(0, |h| h.0),
                    std::ptr::null(),
                    std::ptr::null(),
                    source,
                    destination_address,
                    data,
                    data_len,
                    options,
                    reply_buffer,
                    reply_buffer_len,
                    timeout,
                )
            } else {
                IcmpSendEcho2(
                    handle,
                    event,
                    None,
                    None,
                    destination_address,
                    data,
                    data_len,
                    Some(options),
                    reply_buffer,
                    reply_buffer_len,
                    timeout,
                )
            }
        }
    }

    fn create_raw_reply(&self, reply: *mut u8) -> PingRawReply {
        let reply = unsafe { *(reply as *const ICMP_ECHO_REPLY) };

        // properly handle Network BE
        let addr_ptr = &reply.Address as *const u32 as *const [u8; 4];
        let addr = u32::from_be_bytes(unsafe { *addr_ptr });

        PingRawReply {
            address: IpAddr::V4(Ipv4Addr::from(addr)),
            status: reply.Status,
            rtt: reply.RoundTripTime,
        }
    }
}
