use crate::ctx::Context;
use crate::error::custom_error;
use crate::utils::c_uint_to_u32;

use std::os::raw::c_uint;
use std::{io, mem, net};

#[repr(transparent)]
pub struct GidEntry(ibverbs_sys::ibv_gid_entry);

impl GidEntry {
    #[inline]
    pub fn query(ctx: &Context, port_num: u32, gid_index: u32) -> io::Result<Self> {
        // SAFETY: ffi
        unsafe {
            // TODO: use MaybeUninit to avoid unnecessary initialization?
            let mut gid = mem::MaybeUninit::<Self>::uninit();
            let context = ctx.ffi_ptr();
            let entry = gid.as_mut_ptr().cast::<ibverbs_sys::ibv_gid_entry>();
            let flags = 0; // ASK: what is this?
            let ret = ibverbs_sys::_ibv_query_gid_ex(context, port_num, gid_index, entry, flags);
            if ret != 0 {
                return Err(custom_error("failed to query gid entry"));
            }
            Ok(gid.assume_init())
        }
    }

    #[inline]
    #[must_use]
    pub fn gid_type(&self) -> GidType {
        GidType::from_c_uint(self.0.gid_type)
    }

    #[inline]
    #[must_use]
    pub fn gid(&self) -> Gid {
        Gid(self.0.gid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GidType {
    IB = c_uint_to_u32(ibverbs_sys::IBV_GID_TYPE_IB),
    RoceV1 = c_uint_to_u32(ibverbs_sys::IBV_GID_TYPE_ROCE_V1),
    RoceV2 = c_uint_to_u32(ibverbs_sys::IBV_GID_TYPE_ROCE_V2),
}

impl GidType {
    fn from_c_uint(val: c_uint) -> Self {
        match val {
            ibverbs_sys::IBV_GID_TYPE_IB => GidType::IB,
            ibverbs_sys::IBV_GID_TYPE_ROCE_V1 => GidType::RoceV1,
            ibverbs_sys::IBV_GID_TYPE_ROCE_V2 => GidType::RoceV2,
            _ => panic!("unknown gid type"),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Gid(ibverbs_sys::ibv_gid);

impl Gid {
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(ibverbs_sys::ibv_gid { raw: bytes })
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        // SAFETY: type raw bytes
        unsafe { &self.0.raw }
    }

    #[inline]
    #[must_use]
    pub fn to_ipv6_addr(&self) -> net::Ipv6Addr {
        net::Ipv6Addr::from(*self.as_bytes())
    }

    #[inline]
    #[must_use]
    pub const fn subnet_prefix(&self) -> u64 {
        // SAFETY: POD
        unsafe { self.0.global.subnet_prefix }
    }

    #[inline]
    #[must_use]
    pub const fn interface_id(&self) -> u64 {
        // SAFETY: POD
        unsafe { self.0.global.interface_id }
    }
}

impl PartialEq for Gid {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Gid {}
