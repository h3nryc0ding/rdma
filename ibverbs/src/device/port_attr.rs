use crate::ctx::Context;
use crate::error::from_errno;

use ibverbs_sys::ibv_port_state;
use std::{ffi, io, mem};

pub struct PortAttr(ibverbs_sys::ibv_port_attr);

impl PortAttr {
    #[inline]
    pub fn query(ctx: &Context, port_num: u8) -> io::Result<Self> {
        // SAFETY: ffi
        unsafe {
            let mut port_attr = mem::zeroed();

            let context = ctx.ffi_ptr();
            let ret = ibverbs_sys::ibv_query_port(context, port_num, &mut port_attr);
            if ret != 0 {
                return Err(from_errno(ret));
            }
            Ok(Self(port_attr))
        }
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> PortState {
        PortState::try_from(self.0.state).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn gid_table_len(&self) -> u32 {
        self.0.gid_tbl_len as u32
    }

    #[inline]
    #[must_use]
    pub fn link_layer(&self) -> LinkLayer {
        LinkLayer::try_from(self.0.link_layer as ffi::c_uint).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn lid(&self) -> u16 {
        self.0.lid
    }

    #[inline]
    #[must_use]
    pub fn active_mtu(&self) -> Mtu {
        Mtu::try_from(self.0.active_mtu).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PortState {
    Nop = ibv_port_state::IBV_PORT_NOP,
    Down = ibv_port_state::IBV_PORT_DOWN,
    Init = ibv_port_state::IBV_PORT_INIT,
    Armed = ibv_port_state::IBV_PORT_ARMED,
    Active = ibv_port_state::IBV_PORT_ACTIVE,
    ActiveDefer = ibv_port_state::IBV_PORT_ACTIVE_DEFER,
}

impl TryFrom<ffi::c_uint> for PortState {
    type Error = ();

    fn try_from(value: ffi::c_uint) -> Result<Self, Self::Error> {
        match value {
            ibv_port_state::IBV_PORT_NOP => Ok(PortState::Nop),
            ibv_port_state::IBV_PORT_DOWN => Ok(PortState::Down),
            ibv_port_state::IBV_PORT_INIT => Ok(PortState::Init),
            ibv_port_state::IBV_PORT_ARMED => Ok(PortState::Armed),
            ibv_port_state::IBV_PORT_ACTIVE => Ok(PortState::Active),
            ibv_port_state::IBV_PORT_ACTIVE_DEFER => Ok(PortState::ActiveDefer),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LinkLayer {
    Ethernet = ibverbs_sys::IBV_LINK_LAYER_ETHERNET,
    Infiniband = ibverbs_sys::IBV_LINK_LAYER_INFINIBAND,
    Unspecified = ibverbs_sys::IBV_LINK_LAYER_UNSPECIFIED,
}
impl TryFrom<ffi::c_uint> for LinkLayer {
    type Error = ();

    fn try_from(value: ffi::c_uint) -> Result<Self, Self::Error> {
        match value {
            ibverbs_sys::IBV_LINK_LAYER_ETHERNET => Ok(LinkLayer::Ethernet),
            ibverbs_sys::IBV_LINK_LAYER_INFINIBAND => Ok(LinkLayer::Infiniband),
            ibverbs_sys::IBV_LINK_LAYER_UNSPECIFIED => Ok(LinkLayer::Unspecified),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Mtu {
    Mtu256 = ibverbs_sys::IBV_MTU_256,
    Mtu512 = ibverbs_sys::IBV_MTU_512,
    Mtu1024 = ibverbs_sys::IBV_MTU_1024,
    Mtu2048 = ibverbs_sys::IBV_MTU_2048,
    Mtu4096 = ibverbs_sys::IBV_MTU_4096,
}

impl TryFrom<ffi::c_uint> for Mtu {
    type Error = ();

    fn try_from(value: ffi::c_uint) -> Result<Self, Self::Error> {
        match value {
            ibverbs_sys::IBV_MTU_256 => Ok(Mtu::Mtu256),
            ibverbs_sys::IBV_MTU_512 => Ok(Mtu::Mtu512),
            ibverbs_sys::IBV_MTU_1024 => Ok(Mtu::Mtu1024),
            ibverbs_sys::IBV_MTU_2048 => Ok(Mtu::Mtu2048),
            ibverbs_sys::IBV_MTU_4096 => Ok(Mtu::Mtu4096),
            _ => Err(()),
        }
    }
}

impl Mtu {
    #[inline]
    #[must_use]
    pub fn size(self) -> usize {
        1usize.wrapping_shl((self as u32).wrapping_add(7))
    }
}
