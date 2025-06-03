use ibverbs_sys::{ibv_wc_opcode, ibv_wc_status};
use std::{ffi, fmt, mem};

#[repr(transparent)]
pub struct WorkCompletion(ibverbs_sys::ibv_wc);

impl WorkCompletion {
    #[inline]
    #[must_use]
    pub fn status(&self) -> u32 {
        self.0.status
    }

    #[inline]
    #[must_use]
    pub fn wr_id(&self) -> u64 {
        self.0.wr_id
    }

    #[inline]
    #[must_use]
    pub fn byte_len(&self) -> u32 {
        self.0.byte_len
    }

    #[inline]
    #[must_use]
    pub fn opcode(&self) -> Opcode {
        Opcode::try_from(self.0.opcode).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn imm_data(&self) -> Option<u32> {
        /*
        // SAFETY: tagged union
        unsafe {
            let has_imm = self.0.wc_flags & C::IBV_WC_WITH_IMM != 0;
            has_imm.then_some(self.0.__bindgen_anon_1.imm_data)
        }
        */
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Opcode {
    Send = ibv_wc_opcode::IBV_WC_SEND,
    RdmaWrite = ibv_wc_opcode::IBV_WC_RDMA_WRITE,
    RdmaRead = ibv_wc_opcode::IBV_WC_RDMA_READ,
    CompSwap = ibv_wc_opcode::IBV_WC_COMP_SWAP,
    FetchAdd = ibv_wc_opcode::IBV_WC_FETCH_ADD,
    BindMw = ibv_wc_opcode::IBV_WC_BIND_MW,
    LocalInv = ibv_wc_opcode::IBV_WC_LOCAL_INV,
    Tso = ibv_wc_opcode::IBV_WC_TSO,
    Recv = ibv_wc_opcode::IBV_WC_RECV,
    RecvRdmaWithImm = ibv_wc_opcode::IBV_WC_RECV_RDMA_WITH_IMM,
    TmAdd = ibv_wc_opcode::IBV_WC_TM_ADD,
    TmDel = ibv_wc_opcode::IBV_WC_TM_DEL,
    TmSync = ibv_wc_opcode::IBV_WC_TM_SYNC,
    TmRecv = ibv_wc_opcode::IBV_WC_TM_RECV,
    TmNoTag = ibv_wc_opcode::IBV_WC_TM_NO_TAG,
    Driver1 = ibv_wc_opcode::IBV_WC_DRIVER1,
    Driver2 = ibv_wc_opcode::IBV_WC_DRIVER2,
    Driver3 = ibv_wc_opcode::IBV_WC_DRIVER3,
}

impl TryFrom<ffi::c_uint> for Opcode {
    type Error = ();

    fn try_from(value: ffi::c_uint) -> Result<Self, Self::Error> {
        match value {
            ibv_wc_opcode::IBV_WC_SEND => Ok(Opcode::Send),
            ibv_wc_opcode::IBV_WC_RDMA_WRITE => Ok(Opcode::RdmaWrite),
            ibv_wc_opcode::IBV_WC_RDMA_READ => Ok(Opcode::RdmaRead),
            ibv_wc_opcode::IBV_WC_COMP_SWAP => Ok(Opcode::CompSwap),
            ibv_wc_opcode::IBV_WC_FETCH_ADD => Ok(Opcode::FetchAdd),
            ibv_wc_opcode::IBV_WC_BIND_MW => Ok(Opcode::BindMw),
            ibv_wc_opcode::IBV_WC_LOCAL_INV => Ok(Opcode::LocalInv),
            ibv_wc_opcode::IBV_WC_TSO => Ok(Opcode::Tso),
            ibv_wc_opcode::IBV_WC_RECV => Ok(Opcode::Recv),
            ibv_wc_opcode::IBV_WC_RECV_RDMA_WITH_IMM => Ok(Opcode::RecvRdmaWithImm),
            ibv_wc_opcode::IBV_WC_TM_ADD => Ok(Opcode::TmAdd),
            ibv_wc_opcode::IBV_WC_TM_DEL => Ok(Opcode::TmDel),
            ibv_wc_opcode::IBV_WC_TM_SYNC => Ok(Opcode::TmSync),
            ibv_wc_opcode::IBV_WC_TM_RECV => Ok(Opcode::TmRecv),
            ibv_wc_opcode::IBV_WC_TM_NO_TAG => Ok(Opcode::TmNoTag),
            ibv_wc_opcode::IBV_WC_DRIVER1 => Ok(Opcode::Driver1),
            ibv_wc_opcode::IBV_WC_DRIVER2 => Ok(Opcode::Driver2),
            ibv_wc_opcode::IBV_WC_DRIVER3 => Ok(Opcode::Driver3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WorkCompletionError {
    LocalLength = ibv_wc_status::IBV_WC_LOC_LEN_ERR,
    LocalQPOperation = ibv_wc_status::IBV_WC_LOC_QP_OP_ERR,
    LocalEEContextOperation = ibv_wc_status::IBV_WC_LOC_EEC_OP_ERR,
    LocalProtection = ibv_wc_status::IBV_WC_LOC_PROT_ERR,
    WRFlush = ibv_wc_status::IBV_WC_WR_FLUSH_ERR,
    MWBind = ibv_wc_status::IBV_WC_MW_BIND_ERR,
    BadResponse = ibv_wc_status::IBV_WC_BAD_RESP_ERR,
    LocalAccess = ibv_wc_status::IBV_WC_LOC_ACCESS_ERR,
    RemoteInvalidRequest = ibv_wc_status::IBV_WC_REM_INV_REQ_ERR,
    RemoteAccess = ibv_wc_status::IBV_WC_REM_ACCESS_ERR,
    RemoteOperation = ibv_wc_status::IBV_WC_REM_OP_ERR,
    RetryExceeded = ibv_wc_status::IBV_WC_RETRY_EXC_ERR,
    RnrRetryExceeded = ibv_wc_status::IBV_WC_RNR_RETRY_EXC_ERR,
    LocalRDDViolation = ibv_wc_status::IBV_WC_LOC_RDD_VIOL_ERR,
    RemoteInvalidRDRequest = ibv_wc_status::IBV_WC_REM_INV_RD_REQ_ERR,
    RemoteAborted = ibv_wc_status::IBV_WC_REM_ABORT_ERR,
    InvalidEEContextNumber = ibv_wc_status::IBV_WC_INV_EECN_ERR,
    InvalidEEContextState = ibv_wc_status::IBV_WC_INV_EEC_STATE_ERR,
    Fatal = ibv_wc_status::IBV_WC_FATAL_ERR,
    ResponseTimeout = ibv_wc_status::IBV_WC_RESP_TIMEOUT_ERR,
    General = ibv_wc_status::IBV_WC_GENERAL_ERR,
    TagMatching = ibv_wc_status::IBV_WC_TM_ERR,
    TagMatchingRndvIncomplete = ibv_wc_status::IBV_WC_TM_RNDV_INCOMPLETE,
}

impl WorkCompletionError {
    #[inline]
    pub fn result(status: u32) -> Result<(), WorkCompletionError> {
        let status = status;
        if status == ibv_wc_status::IBV_WC_SUCCESS {
            Ok(())
        } else {
            Err(WorkCompletionError::from(status))
        }
    }
}

impl From<ffi::c_uint> for WorkCompletionError {
    fn from(value: ffi::c_uint) -> Self {
        // SAFETY: continuous integer enum
        unsafe { mem::transmute(value as u32) }
    }
}

impl fmt::Display for WorkCompletionError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f) // TODO: error message
    }
}

impl std::error::Error for WorkCompletionError {}
