use crate::error::create_resource;
use crate::pd::ProtectionDomain;

use std::{ffi, io, ptr, sync};

#[derive(Clone)]
pub struct MemoryWindow(sync::Arc<Owner>);

impl MemoryWindow {
    #[inline]
    pub fn alloc(pd: &ProtectionDomain, mw_type: MemoryWindowType) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let mw_type = mw_type as ffi::c_int;
            let mw = create_resource(
                || ibverbs_sys::ibv_alloc_mw(pd.ffi_ptr(), mw_type),
                || "failed to allocate memory window",
            )?;
            sync::Arc::new(Owner {
                mw,
                _pd: pd.clone(),
            })
        };
        Ok(Self(owner))
    }
}

struct Owner {
    mw: ptr::NonNull<ibverbs_sys::ibv_mw>,
    _pd: ProtectionDomain,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_mw {
        self.mw.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let mw = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_dealloc_mw(mw);
            assert_eq!(ret, 0);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryWindowType {
    Type1 = ibverbs_sys::IBV_MW_TYPE_1,
    Type2 = ibverbs_sys::IBV_MW_TYPE_2,
}
