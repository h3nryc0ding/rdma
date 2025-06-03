use crate::device::Device;
use crate::error::create_resource;

use std::{io, ptr, sync};

#[derive(Clone)]
pub struct Context(sync::Arc<Owner>);

impl Context {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_context {
        self.0.ffi_ptr()
    }

    #[inline]
    pub fn open(device: &Device) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let ctx = create_resource(
                || ibverbs_sys::ibv_open_device(device.ffi_ptr()),
                || "failed to open device",
            )?;
            sync::Arc::new(Owner { ctx })
        };
        Ok(Self(owner))
    }
}

struct Owner {
    ctx: ptr::NonNull<ibverbs_sys::ibv_context>,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_context {
        self.ctx.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let context = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_close_device(context);
            assert_eq!(ret, 0);
        }
    }
}
