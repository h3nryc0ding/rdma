use crate::error::create_resource;
use crate::pd::ProtectionDomain;
use crate::utils::ptr_to_addr;

use std::{ffi, io, ptr, sync};

use ibverbs_sys::ibv_access_flags;
use numeric_cast::NumericCast;

#[derive(Clone)]
pub struct MemoryRegion<T = ()>(sync::Arc<Owner<T>>);

impl<T> MemoryRegion<T> {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_mr {
        self.0.ffi_ptr()
    }

    /// Registers a memory region associated with the protection domain `pd`.
    /// The memory region's starting address is `addr` and its size is `length`.
    ///
    /// # Safety
    /// 1. the memory region must be valid until it is deregistered
    /// 2. the memory region must be initialized before it is read for the first time
    #[allow(clippy::arc_with_non_send_sync)] // FIXME: false positive
    #[inline]
    pub unsafe fn register(
        pd: &ProtectionDomain,
        addr: *mut u8,
        length: usize,
        access_flags: AccessFlags,
        metadata: T,
    ) -> io::Result<Self> {
        let owner = {
            let addr = addr.cast();
            let access_flags = access_flags.bits() as ffi::c_int;
            let mr = create_resource(
                || ibverbs_sys::ibv_reg_mr(pd.ffi_ptr(), addr, length, access_flags),
                || "failed to register memory region",
            )?;
            sync::Arc::new(Owner {
                mr,
                metadata,
                _pd: pd.clone(),
            })
        };
        Ok(Self(owner))
    }

    #[inline]
    #[must_use]
    pub fn lkey(&self) -> u32 {
        let mr = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*mr).lkey }
    }

    #[inline]
    #[must_use]
    pub fn rkey(&self) -> u32 {
        let mr = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*mr).rkey }
    }

    #[inline]
    #[must_use]
    pub fn addr_ptr(&self) -> *mut u8 {
        let mr = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*mr).addr.cast() }
    }

    #[inline]
    #[must_use]
    pub fn addr_u64(&self) -> u64 {
        let mr = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { ptr_to_addr((*mr).addr) }.numeric_cast()
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> usize {
        let mr = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*mr).length }
    }

    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &T {
        self.0.metadata()
    }
}

struct Owner<T> {
    mr: ptr::NonNull<ibverbs_sys::ibv_mr>,

    metadata: T,

    _pd: ProtectionDomain,
}

/// SAFETY: owned type
unsafe impl<T: Send> Send for Owner<T> {}
/// SAFETY: owned type
unsafe impl<T: Sync> Sync for Owner<T> {}

impl<T> Owner<T> {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_mr {
        self.mr.as_ptr()
    }

    fn metadata(&self) -> &T {
        &self.metadata
    }
}

impl<T> Drop for Owner<T> {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let mr = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_dereg_mr(mr);
            assert_eq!(ret, 0);
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct AccessFlags: u32 {
            const LOCAL_WRITE       = ibv_access_flags::IBV_ACCESS_LOCAL_WRITE.0;
            const REMOTE_WRITE      = ibv_access_flags::IBV_ACCESS_REMOTE_WRITE.0;
            const REMOTE_READ       = ibv_access_flags::IBV_ACCESS_REMOTE_READ.0;
            const REMOTE_ATOMIC     = ibv_access_flags::IBV_ACCESS_REMOTE_ATOMIC.0;
            const MW_BIND           = ibv_access_flags::IBV_ACCESS_MW_BIND.0;
            const ZERO_BASED        = ibv_access_flags::IBV_ACCESS_ZERO_BASED.0;
            const ON_DEMAND         = ibv_access_flags::IBV_ACCESS_ON_DEMAND.0;
            const HUGETLB           = ibv_access_flags::IBV_ACCESS_HUGETLB.0;
            const RELAXED_ORDERING  = ibv_access_flags::IBV_ACCESS_RELAXED_ORDERING.0;
        }
}
