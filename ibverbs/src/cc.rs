use crate::cq::{self, CompletionQueue};
use crate::ctx::Context;
use crate::error::{create_resource, custom_error};
use crate::weakset::WeakSet;

use std::os::raw::c_void;
use std::os::unix::prelude::{AsRawFd, RawFd};
use std::{io, ptr, sync};

#[derive(Clone)]
pub struct CompChannel(sync::Arc<Owner>);

impl CompChannel {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_comp_channel {
        self.0.ffi_ptr()
    }

    #[inline]
    pub fn create(ctx: &Context) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let cc = create_resource(
                || ibverbs_sys::ibv_create_comp_channel(ctx.ffi_ptr()),
                || "failed to create completion channel",
            )?;

            sync::Arc::new(Owner {
                cc,
                cq_ref: sync::Mutex::new(WeakSet::new()),
                _ctx: ctx.clone(),
            })
        };
        Ok(Self(owner))
    }

    #[inline]
    pub fn wait_cq_event(&self) -> io::Result<CompletionQueue> {
        let cc = self.ffi_ptr();
        let mut cq: *mut ibverbs_sys::ibv_cq = ptr::null_mut();
        let mut cq_context: *mut c_void = ptr::null_mut();
        // SAFETY: ffi
        unsafe {
            let ret = ibverbs_sys::ibv_get_cq_event(cc, &mut cq, &mut cq_context);
            if ret != 0 {
                return Err(custom_error("failed to get completion event"));
            }
            debug_assert_eq!((*cq).cq_context, cq_context);
        }
        // SAFETY:
        // 1. the cq is associated with the cc
        // 2. the cc is holding a weak reference to the cq
        // 3. here may panic because the cq may have been destroyed
        unsafe { Ok(CompletionQueue::from_cq_context(cq_context)) }
    }

    pub(crate) fn add_cq_ref(&self, cq: sync::Weak<cq::Owner>) {
        let mut guard = self.0.cq_ref.lock().unwrap();
        guard.insert(cq);
    }

    pub(crate) fn del_cq_ref(&self, cq: &cq::Owner) -> bool {
        let mut guard = self.0.cq_ref.lock().unwrap();
        guard.remove(cq)
    }
}

impl AsRawFd for CompChannel {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        let cc = self.ffi_ptr();
        // SAFETY: reading a immutable field of a concurrent ffi type
        unsafe { (*cc).fd }
    }
}

struct Owner {
    cc: ptr::NonNull<ibverbs_sys::ibv_comp_channel>,

    cq_ref: sync::Mutex<WeakSet<cq::Owner>>,
    _ctx: Context,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_comp_channel {
        self.cc.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let cc = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_destroy_comp_channel(cc);
            assert_eq!(ret, 0);
        }
    }
}
