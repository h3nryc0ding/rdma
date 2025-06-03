use crate::ah::AddressHandle;
use crate::utils::ptr_as_mut;

use ibverbs_sys::{ibv_send_flags, ibv_wr_opcode};
use std::{ffi, mem};

#[repr(transparent)]
pub struct SendRequest(ibverbs_sys::ibv_send_wr);

/// SAFETY: ffi pointer data
/// the actual usage is unsafe (`C::ibv_post_send`)
unsafe impl Send for SendRequest {}
/// SAFETY: ffi pointer data
/// the actual usage is unsafe (`C::ibv_post_send`)
unsafe impl Sync for SendRequest {}

#[repr(transparent)]
pub struct RecvRequest(ibverbs_sys::ibv_recv_wr);

/// SAFETY: ffi pointer data
/// the actual usage is unsafe (`C::ibv_post_recv`)
unsafe impl Send for RecvRequest {}
/// SAFETY: ffi pointer data
/// the actual usage is unsafe (`C::ibv_post_recv`)
unsafe impl Sync for RecvRequest {}

#[repr(C)]
pub struct Sge {
    pub addr: u64,
    pub length: u32,
    pub lkey: u32,
}

/// SAFETY: ffi pointer data
/// the actual usage is unsafe
unsafe impl Send for Sge {}
/// SAFETY: ffi pointer data
/// the actual usage is unsafe
unsafe impl Sync for Sge {}

impl SendRequest {
    #[inline]
    #[must_use]
    pub fn zeroed() -> Self {
        // SAFETY: POD ffi type
        unsafe { Self(mem::zeroed()) }
    }

    #[inline]
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.0.wr_id = id;
        self
    }

    #[inline]
    pub fn next(&mut self, next: *mut Self) -> &mut Self {
        self.0.next = next.cast();
        self
    }

    #[inline]
    pub fn sg_list(&mut self, sg_list: &[Sge]) -> &mut Self {
        self.0.num_sge = sg_list.len() as ffi::c_int;
        self.0.sg_list = ptr_as_mut(sg_list.as_ptr()).cast::<ibverbs_sys::ibv_sge>();
        self
    }

    #[inline]
    pub fn opcode(&mut self, opcode: Opcode) -> &mut Self {
        self.0.opcode = opcode as ffi::c_uint;
        self
    }

    #[inline]
    pub fn send_flags(&mut self, send_flags: SendFlags) -> &mut Self {
        self.0.send_flags = send_flags.bits();
        self
    }

    #[inline]
    pub unsafe fn ud_ah(&mut self, ah: &AddressHandle) -> &mut Self {
        self.0.wr.ud.ah = ah.ffi_ptr();
        self
    }

    #[inline]
    pub unsafe fn ud_remote_qpn(&mut self, remote_qpn: u32) -> &mut Self {
        self.0.wr.ud.remote_qpn = remote_qpn;
        self
    }

    #[inline]
    pub unsafe fn ud_remote_qkey(&mut self, remote_qkey: u32) -> &mut Self {
        self.0.wr.ud.remote_qkey = remote_qkey;
        self
    }

    #[inline]
    pub unsafe fn rdma_remote_addr(&mut self, remote_addr: u64) -> &mut Self {
        self.0.wr.rdma.remote_addr = remote_addr;
        self
    }

    #[inline]
    pub unsafe fn rdma_rkey(&mut self, rkey: u32) -> &mut Self {
        self.0.wr.rdma.rkey = rkey;
        self
    }

    #[inline]
    pub fn imm_data(&mut self, imm_data: u32) -> &mut Self {
        self.0.__bindgen_anon_1.imm_data = imm_data;
        self
    }
}

impl RecvRequest {
    #[inline]
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.0.wr_id = id;
        self
    }

    #[inline]
    pub fn next(&mut self, next: *mut Self) -> &mut Self {
        self.0.next = next.cast();
        self
    }

    #[inline]
    pub fn sg_list(&mut self, sg_list: &[Sge]) -> &mut Self {
        self.0.num_sge = sg_list.len() as ffi::c_int;
        self.0.sg_list = ptr_as_mut(sg_list.as_ptr()).cast::<ibverbs_sys::ibv_sge>();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Opcode {
    Send = ibv_wr_opcode::IBV_WR_SEND as ffi::c_uint,
    SendWithImm = ibv_wr_opcode::IBV_WR_SEND_WITH_IMM as ffi::c_uint,
    Write = ibv_wr_opcode::IBV_WR_RDMA_WRITE as ffi::c_uint,
    Read = ibv_wr_opcode::IBV_WR_RDMA_READ as ffi::c_uint,
    AtomicFetchAdd = ibv_wr_opcode::IBV_WR_ATOMIC_FETCH_AND_ADD as ffi::c_uint,
    AtomicCAS = ibv_wr_opcode::IBV_WR_ATOMIC_CMP_AND_SWP as ffi::c_uint,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SendFlags: u32 {
        const FENCE = ibv_send_flags::IBV_SEND_FENCE.0;
        const SIGNALED = ibv_send_flags::IBV_SEND_SIGNALED.0;
        const SOLICITED = ibv_send_flags::IBV_SEND_SOLICITED.0;
        const INLINE = ibv_send_flags::IBV_SEND_INLINE.0;
        const IP_CSUM = ibv_send_flags::IBV_SEND_IP_CSUM.0;
    }
}
