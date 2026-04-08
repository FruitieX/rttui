use paste::paste;

pub(crate) const ICMP_HEADER_SIZE: usize = 8;

#[repr(C)]
pub(crate) struct IcmpEchoHeader {
    pub r#type: u8,
    pub code: u8,
    checksum: [u8; 2],
    ident: [u8; 2],
    seq: [u8; 2],
}

macro_rules! simple_property {
    ($type:ty | $name:ident) => {
        paste! {
            pub(crate) fn $name(&self) -> $type {
                $type::from_be_bytes(self.$name)
            }
            pub(crate) fn [<set_ $name>](&mut self, $name: $type) {
                self.$name = $name.to_be_bytes();
            }
        }
    };
}

impl IcmpEchoHeader {
    #![allow(dead_code)]

    pub(crate) fn get_mut_ref(be_buffer: &mut [u8]) -> &mut IcmpEchoHeader {
        let header = be_buffer.as_mut_ptr() as *mut IcmpEchoHeader;
        unsafe { &mut *header }
    }

    pub(crate) fn get_ref(be_buffer: &[u8]) -> &IcmpEchoHeader {
        let header = be_buffer.as_ptr() as *mut IcmpEchoHeader;
        unsafe { &*header }
    }

    simple_property![u16 | checksum];
    simple_property![u16 | ident];
    simple_property![u16 | seq];
}
