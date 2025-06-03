use crate::ctx::Context;
use crate::error::create_resource;

use std::{io, mem, ptr, sync};

#[derive(Clone)]
pub struct DeviceMemory(sync::Arc<Owner>);

impl DeviceMemory {
    #[inline]
    #[must_use]
    pub fn options() -> DeviceMemoryOptions {
        DeviceMemoryOptions::default()
    }

    #[inline]
    pub fn alloc(ctx: &Context, mut options: DeviceMemoryOptions) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let attr = &mut options.attr;
            let dm = create_resource(
                || ibverbs_sys::ibv_alloc_dm(ctx.ffi_ptr(), attr),
                || "failed to allocate device memory",
            )?;
            sync::Arc::new(Owner {
                dm,
                _ctx: ctx.clone(),
            })
        };
        Ok(Self(owner))
    }
}

struct Owner {
    dm: ptr::NonNull<ibverbs_sys::ibv_dm>,
    _ctx: Context,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_dm {
        self.dm.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let dm = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_free_dm(dm);
            assert_eq!(ret, 0);
        }
    }
}

pub struct DeviceMemoryOptions {
    attr: ibverbs_sys::ibv_alloc_dm_attr,
}

impl Default for DeviceMemoryOptions {
    #[inline]
    fn default() -> Self {
        Self {
            // SAFETY: POD ffi type
            attr: unsafe { mem::zeroed() },
        }
    }
}
