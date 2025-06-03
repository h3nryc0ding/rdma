use crate::ctx::Context;
use crate::error::create_resource;

use std::{io, ptr, sync};

#[derive(Clone)]
pub struct ProtectionDomain(sync::Arc<Owner>);

impl ProtectionDomain {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_pd {
        self.0.ffi_ptr()
    }

    #[inline]
    pub fn alloc(ctx: &Context) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let pd = create_resource(
                || ibverbs_sys::ibv_alloc_pd(ctx.ffi_ptr()),
                || "failed to allocate protection domain",
            )?;
            sync::Arc::new(Owner {
                pd,
                _ctx: ctx.clone(),
            })
        };
        Ok(Self(owner))
    }
}

struct Owner {
    pd: ptr::NonNull<ibverbs_sys::ibv_pd>,

    _ctx: Context,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_pd {
        self.pd.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let pd = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_dealloc_pd(pd);
            assert_eq!(ret, 0);
        }
    }
}
