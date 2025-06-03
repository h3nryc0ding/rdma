use crate::ah::AddressHandleOptions;
use crate::cq::CompletionQueue;
use crate::ctx::Context;
use crate::device::Mtu;
use crate::error::{create_resource, from_errno, get_errno, set_errno};
use crate::mr::AccessFlags;
use crate::pd::ProtectionDomain;
use crate::srq::SharedReceiveQueue;
use crate::utils::ptr_as_mut;
use crate::utils::{usize_to_void_ptr, void_ptr_to_usize};
use crate::wr::{RecvRequest, SendRequest};

use ibverbs_sys::{ibv_qp_attr_mask, ibv_qp_state};
use std::{ffi, io, mem, ptr, sync};

#[derive(Clone)]
pub struct QueuePair(sync::Arc<Owner>);

impl QueuePair {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_qp {
        self.0.ffi_ptr()
    }

    #[inline]
    #[must_use]
    pub fn options() -> QueuePairOptions {
        QueuePairOptions::default()
    }

    #[inline]
    pub fn create(ctx: &Context, mut options: QueuePairOptions) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let context = ctx.ffi_ptr();
            let qp_attr = &mut options.attr;

            let qp = create_resource(
                || ibverbs_sys::ibv_create_qp_ex(context, qp_attr),
                || "failed to create queue pair",
            )?;

            sync::Arc::new(Owner {
                qp,
                _pd: options.pd,
                send_cq: options.send_cq,
                recv_cq: options.recv_cq,
                _srq: options.srq,
            })
        };
        Ok(Self(owner))
    }

    #[inline]
    #[must_use]
    pub fn qp_num(&self) -> u32 {
        let qp = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*qp).qp_num }
    }

    #[inline]
    #[must_use]
    pub fn user_data(&self) -> usize {
        let qp = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { void_ptr_to_usize((*qp).qp_context) }
    }

    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn post_send(&self, send_wr: &SendRequest) -> io::Result<()> {
        let qp = self.ffi_ptr();
        let wr: *mut ibverbs_sys::ibv_send_wr = ptr_as_mut(send_wr).cast();
        let mut bad_wr: *mut ibverbs_sys::ibv_send_wr = ptr::null_mut();
        set_errno(0);
        let ret = ibverbs_sys::ibv_post_send(qp, wr, &mut bad_wr);
        if ret != 0 {
            let errno = get_errno();
            if errno != 0 {
                return Err(from_errno(errno));
            }
            return Err(from_errno(ret.abs()));
        }
        Ok(())
    }

    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn post_recv(&self, recv_wr: &RecvRequest) -> io::Result<()> {
        let qp = self.ffi_ptr();
        let wr: *mut ibverbs_sys::ibv_recv_wr = ptr_as_mut(recv_wr).cast();
        let mut bad_wr: *mut ibverbs_sys::ibv_recv_wr = ptr::null_mut();
        set_errno(0);
        let ret = ibverbs_sys::ibv_post_recv(qp, wr, &mut bad_wr);
        if ret != 0 {
            let errno = get_errno();
            if errno != 0 {
                return Err(from_errno(errno));
            }
            return Err(from_errno(ret.abs()));
        }
        Ok(())
    }

    #[inline]
    pub fn modify(&self, mut options: ModifyOptions) -> io::Result<()> {
        let qp = self.ffi_ptr();
        // SAFETY: ffi
        unsafe {
            let attr_mask = mem::transmute(options.mask);
            let attr = options.attr.as_mut_ptr();
            let ret = ibverbs_sys::ibv_modify_qp(qp, attr, attr_mask);
            if ret != 0 {
                return Err(from_errno(ret));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn query(&self, options: QueryOptions) -> io::Result<QueuePairAttr> {
        let qp = self.ffi_ptr();
        // SAFETY: ffi
        unsafe {
            let attr_mask = mem::transmute(options.mask);
            let mut attr: QueuePairAttr = mem::zeroed();
            let mut init_attr: ibverbs_sys::ibv_qp_init_attr = mem::zeroed();
            let ret = ibverbs_sys::ibv_query_qp(qp, &mut attr.attr, attr_mask, &mut init_attr);
            if ret != 0 {
                return Err(from_errno(ret));
            }
            attr.mask = options.mask;
            Ok(attr)
        }
    }

    #[inline]
    #[must_use]
    pub fn send_cq(&self) -> Option<&CompletionQueue> {
        self.0.send_cq.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn recv_cq(&self) -> Option<&CompletionQueue> {
        self.0.recv_cq.as_ref()
    }
}

struct Owner {
    qp: ptr::NonNull<ibverbs_sys::ibv_qp>,

    _pd: Option<ProtectionDomain>,
    send_cq: Option<CompletionQueue>,
    recv_cq: Option<CompletionQueue>,
    _srq: Option<SharedReceiveQueue>,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_qp {
        self.qp.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let qp: *mut ibverbs_sys::ibv_qp = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_destroy_qp(qp);
            assert_eq!(ret, 0);
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct QueuePairCapacity {
    pub max_send_wr: u32,
    pub max_recv_wr: u32,
    pub max_send_sge: u32,
    pub max_recv_sge: u32,
    pub max_inline_data: u32,
}

impl Default for QueuePairCapacity {
    #[inline]
    fn default() -> Self {
        // SAFETY: POD ffi type
        unsafe { mem::zeroed() }
    }
}

impl QueuePairCapacity {
    fn into_ctype(self) -> ibverbs_sys::ibv_qp_cap {
        // SAFETY: same repr
        unsafe { mem::transmute(self) }
    }
    fn from_ctype_ref(cap: &ibverbs_sys::ibv_qp_cap) -> &Self {
        // SAFETY: same repr
        unsafe { mem::transmute(cap) }
    }
}

pub struct QueuePairOptions {
    attr: ibverbs_sys::ibv_qp_init_attr_ex,

    send_cq: Option<CompletionQueue>,
    recv_cq: Option<CompletionQueue>,
    pd: Option<ProtectionDomain>,
    srq: Option<SharedReceiveQueue>,
}

// SAFETY: owned type
unsafe impl Send for QueuePairOptions {}
// SAFETY: owned type
unsafe impl Sync for QueuePairOptions {}

impl Default for QueuePairOptions {
    #[inline]
    fn default() -> Self {
        Self {
            // SAFETY: POD ffi type
            attr: unsafe { mem::zeroed() },
            send_cq: None,
            recv_cq: None,
            pd: None,
            srq: None,
        }
    }
}

impl QueuePairOptions {
    #[inline]
    pub fn user_data(&mut self, user_data: usize) -> &mut Self {
        self.attr.qp_context = usize_to_void_ptr(user_data);
        self
    }

    #[inline]
    pub fn send_cq(&mut self, send_cq: &CompletionQueue) -> &mut Self {
        self.attr.send_cq = ibverbs_sys::ibv_cq_ex_to_cq(send_cq.ffi_ptr());
        self.send_cq = Some(send_cq.clone());
        self
    }

    #[inline]
    pub fn recv_cq(&mut self, recv_cq: &CompletionQueue) -> &mut Self {
        if self.srq.take().is_some() {
            self.attr.srq = ptr::null_mut();
        }
        self.attr.recv_cq = ibverbs_sys::ibv_cq_ex_to_cq(recv_cq.ffi_ptr());
        self.recv_cq = Some(recv_cq.clone());
        self
    }

    #[inline]
    pub fn qp_type(&mut self, qp_type: QueuePairType) -> &mut Self {
        self.attr.qp_type = qp_type as ffi::c_uint;
        self
    }

    #[inline]
    pub fn sq_sig_all(&mut self, sq_sig_all: bool) -> &mut Self {
        self.attr.sq_sig_all = sq_sig_all as ffi::c_int;
        self
    }

    #[inline]
    pub fn cap(&mut self, cap: QueuePairCapacity) -> &mut Self {
        self.attr.cap = cap.into_ctype();
        self
    }

    #[inline]
    pub fn pd(&mut self, pd: &ProtectionDomain) -> &mut Self {
        self.attr.pd = pd.ffi_ptr();
        self.attr.comp_mask |= ibverbs_sys::IBV_QP_INIT_ATTR_PD;
        self.pd = Some(pd.clone());
        self
    }

    #[inline]
    pub fn srq(&mut self, srq: &SharedReceiveQueue) -> &mut Self {
        if self.recv_cq.take().is_some() {
            self.attr.recv_cq = ptr::null_mut();
        }
        self.attr.srq = srq.ffi_ptr();
        self.srq = Some(srq.clone());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QueuePairType {
    RC = ibverbs_sys::ibv_qp_type::IBV_QPT_RC,
    UC = ibverbs_sys::ibv_qp_type::IBV_QPT_UC,
    UD = ibverbs_sys::ibv_qp_type::IBV_QPT_UD,
    Driver = ibverbs_sys::ibv_qp_type::IBV_QPT_DRIVER,
    XrcRecv = ibverbs_sys::ibv_qp_type::IBV_QPT_XRC_RECV,
    XrcSend = ibverbs_sys::ibv_qp_type::IBV_QPT_XRC_SEND,
}

#[repr(C)]
pub struct ModifyOptions {
    mask: ibverbs_sys::ibv_qp_attr_mask,
    attr: mem::MaybeUninit<ibverbs_sys::ibv_qp_attr>,
}

// SAFETY: owned type
unsafe impl Send for ModifyOptions {}
// SAFETY: owned type
unsafe impl Sync for ModifyOptions {}

impl Default for ModifyOptions {
    #[inline]
    fn default() -> Self {
        Self {
            mask: 0,
            attr: mem::MaybeUninit::uninit(),
        }
    }
}

macro_rules! modify_option {
    ($mask: ident, $field: ident, $ty: ty, $($cvt:tt)+) => {
        #[inline]
        pub fn $field(&mut self, $field: $ty) -> &mut Self {
            // SAFETY: write uninit field
            unsafe {
                let attr = self.attr.as_mut_ptr();
                let p = ptr::addr_of_mut!((*attr).$field);
                p.write($($cvt)+);
            }
            self.mask |= ibverbs_sys::$mask;
            self
        }
    };
}

impl ModifyOptions {
    modify_option!(IBV_QP_STATE, qp_state, QueuePairState, qp_state.to_c_uint());
    modify_option!(IBV_QP_PKEY_INDEX, pkey_index, u16, pkey_index);
    modify_option!(IBV_QP_PORT, port_num, u8, port_num);
    modify_option!(IBV_QP_QKEY, qkey, u32, qkey);
    modify_option!(
        IBV_QP_ACCESS_FLAGS,
        qp_access_flags,
        AccessFlags,
        qp_access_flags.to_c_uint()
    );
    modify_option!(IBV_QP_PATH_MTU, path_mtu, Mtu, path_mtu.to_c_uint());
    modify_option!(IBV_QP_DEST_QPN, dest_qp_num, u32, dest_qp_num);
    modify_option!(IBV_QP_RQ_PSN, rq_psn, u32, rq_psn);
    modify_option!(
        IBV_QP_MAX_DEST_RD_ATOMIC,
        max_dest_rd_atomic,
        u8,
        max_dest_rd_atomic
    );
    modify_option!(IBV_QP_MIN_RNR_TIMER, min_rnr_timer, u8, min_rnr_timer);
    modify_option!(
        IBV_QP_AV,
        ah_attr,
        AddressHandleOptions,
        ah_attr.into_ctype()
    );
    modify_option!(IBV_QP_TIMEOUT, timeout, u8, timeout);
    modify_option!(IBV_QP_RETRY_CNT, retry_cnt, u8, retry_cnt);
    modify_option!(IBV_QP_RNR_RETRY, rnr_retry, u8, rnr_retry);
    modify_option!(IBV_QP_SQ_PSN, sq_psn, u32, sq_psn);
    modify_option!(IBV_QP_MAX_QP_RD_ATOMIC, max_rd_atomic, u8, max_rd_atomic);
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct QueryOptions {
    mask: ibverbs_sys::ibv_qp_attr_mask,
}

impl Default for QueryOptions {
    #[inline]
    fn default() -> Self {
        // SAFETY: POD ffi type
        unsafe { mem::zeroed() }
    }
}

impl QueryOptions {
    #[inline]
    pub fn cap(&mut self) -> &mut Self {
        self.mask |= ibverbs_sys::IBV_QP_CAP;
        self
    }

    #[inline]
    pub fn qp_state(&mut self) -> &mut Self {
        self.mask |= ibverbs_sys::IBV_QP_STATE;
        self
    }
}

#[repr(C)]
pub struct QueuePairAttr {
    mask: ibverbs_sys::ibv_qp_attr_mask,
    attr: ibverbs_sys::ibv_qp_attr,
}

// SAFETY: owned type
unsafe impl Send for QueuePairAttr {}
// SAFETY: owned type
unsafe impl Sync for QueuePairAttr {}

impl QueuePairAttr {
    #[inline]
    #[must_use]
    pub fn cap(&self) -> Option<&QueuePairCapacity> {
        (self.mask & ibverbs_sys::IBV_QP_CAP != 0)
            .then(|| QueuePairCapacity::from_ctype_ref(&self.attr.cap))
    }

    #[inline]
    #[must_use]
    pub fn qp_state(&self) -> Option<QueuePairState> {
        if self.mask & ibv_qp_attr_mask::IBV_QP_STATE == ibv_qp_attr_mask(0) {
            None
        } else {
            Some(QueuePairState::try_from(self.attr.qp_state).unwrap())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QueuePairState {
    Reset = ibv_qp_state::IBV_QPS_RESET,
    Initialize = ibv_qp_state::IBV_QPS_INIT,
    ReadyToReceive = ibv_qp_state::IBV_QPS_RTR,
    ReadyToSend = ibv_qp_state::IBV_QPS_RTS,
    SendQueueDrained = ibv_qp_state::IBV_QPS_SQD,
    SendQueueError = ibv_qp_state::IBV_QPS_SQE,
    Error = ibv_qp_state::IBV_QPS_ERR,
    Unknown = ibv_qp_state::IBV_QPS_UNKNOWN, // ASK: what is this
}

impl TryFrom<ffi::c_uint> for QueuePairState {
    type Error = ();

    fn try_from(value: ffi::c_uint) -> Result<Self, Self::Error> {
        match value {
            ibv_qp_state::IBV_QPS_RESET => Ok(QueuePairState::Reset),
            ibv_qp_state::IBV_QPS_INIT => Ok(QueuePairState::Initialize),
            ibv_qp_state::IBV_QPS_RTR => Ok(QueuePairState::ReadyToReceive),
            ibv_qp_state::IBV_QPS_RTS => Ok(QueuePairState::ReadyToSend),
            ibv_qp_state::IBV_QPS_SQD => Ok(QueuePairState::SendQueueDrained),
            ibv_qp_state::IBV_QPS_SQE => Ok(QueuePairState::SendQueueError),
            ibv_qp_state::IBV_QPS_ERR => Ok(QueuePairState::Error),
            ibv_qp_state::IBV_QPS_UNKNOWN => Ok(QueuePairState::Unknown),
            _ => Err(()),
        }
    }
}
