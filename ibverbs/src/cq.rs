use crate::ctx::Context;
use crate::error::{create_resource, from_errno};
use crate::utils::ptr_as_mut;

use crate::cc::CompChannel;
use crate::wc::WorkCompletion;
use std::{
    ffi, io, mem, os, ptr, slice,
    sync::{self, atomic},
};

#[derive(Clone)]
pub struct CompletionQueue(sync::Arc<Owner>);

impl CompletionQueue {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_cq_ex {
        self.0.ffi_ptr()
    }

    #[inline]
    pub fn create(ctx: &Context, options: CompletionQueueOptions) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let context = ctx.ffi_ptr();

            let mut cq_attr: ibverbs_sys::ibv_cq_init_attr_ex = mem::zeroed();
            cq_attr.cqe = options.cqe as u32;

            if let Some(ref cc) = options.channel {
                cq_attr.channel = cc.ffi_ptr();
            }

            let cq = create_resource(
                || ibverbs_sys::ibv_create_cq_ex(context, &mut cq_attr),
                || "failed to create completion queue",
            )?;

            sync::Arc::new(Owner {
                cq,
                user_data: options.user_data,
                comp_events_completed: sync::atomic::AtomicU32::new(0),
                _ctx: ctx.clone(),
                cc: options.channel,
            })
        };

        if let Some(ref cc) = owner.cc {
            cc.add_cq_ref(sync::Arc::downgrade(&owner));
        }

        // SAFETY: setup self-reference in cq_context
        unsafe {
            let owner_ptr: *const Owner = &*owner;
            let cq = owner.ffi_ptr();
            (*cq).cq_context = ptr_as_mut(owner_ptr).cast();
        }

        Ok(Self(owner))
    }

    /// # Panics
    /// + if the completion queue has been destroyed
    ///
    /// # SAFETY
    /// 1. `cq_context` must come from the pointee of `CompletionQueue::ffi_ptr`
    /// 2. there must be at least one weak reference to the completion queue owner
    pub(crate) unsafe fn from_cq_context(cq_context: *mut ffi::c_void) -> Self {
        let owner_ptr: *const Owner = cq_context.cast();
        let weak = mem::ManuallyDrop::new(sync::Weak::from_raw(owner_ptr));
        let owner = sync::Weak::upgrade(&weak).expect("the completion queue has been destroyed");
        Self(owner)
    }

    #[inline]
    #[must_use]
    pub fn user_data(&self) -> usize {
        self.0.user_data
    }

    fn req_notify(&self, solicited_only: bool) -> io::Result<()> {
        let cq = self.ffi_ptr();
        // SAFETY: ffi
        let ret = unsafe {
            let solicited_only = solicited_only as ffi::c_int;
            ibverbs_sys::ibv_req_notify_cq(ibverbs_sys::ibv_cq_ex_to_cq(cq), solicited_only)
        };
        if ret != 0 {
            return Err(from_errno(ret));
        }
        Ok(())
    }

    #[inline]
    pub fn req_notify_all(&self) -> io::Result<()> {
        self.req_notify(false)
    }

    #[inline]
    pub fn req_notify_solicited(&self) -> io::Result<()> {
        self.req_notify(true)
    }

    #[inline]
    pub fn ack_cq_events(&self, cnt: u32) {
        self.0
            .comp_events_completed
            .fetch_add(cnt, atomic::Ordering::Relaxed);
    }

    #[inline]
    pub fn poll<'wc>(
        &self,
        buf: &'wc mut [mem::MaybeUninit<WorkCompletion>],
    ) -> io::Result<&'wc mut [WorkCompletion]> {
        // SAFETY: ffi
        unsafe {
            let num_entries = buf.len() as ffi::c_int;
            let wc = buf.as_mut_ptr().cast::<ibverbs_sys::ibv_wc>();
            let cq = ibverbs_sys::ibv_cq_ex_to_cq(self.ffi_ptr());
            let ret = ibverbs_sys::ibv_poll_cq(cq, num_entries, wc);
            if ret < 0 {
                return Err(from_errno(ret.wrapping_neg()));
            }
            let len: usize = ret.numeric_cast();
            let data = wc.cast::<WorkCompletion>();
            Ok(slice::from_raw_parts_mut(data, len))
        }
    }
}

pub(crate) struct Owner {
    cq: ptr::NonNull<ibverbs_sys::ibv_cq_ex>,
    user_data: usize,
    comp_events_completed: atomic::AtomicU32,

    cc: Option<CompChannel>,
    _ctx: Context,
}

/// SAFETY: owned type
unsafe impl Send for Owner {}
/// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_cq_ex {
        self.cq.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        if let Some(ref cc) = self.cc {
            assert!(cc.del_cq_ref(self));
        }

        // SAFETY: ffi
        unsafe {
            let cq = ibverbs_sys::ibv_cq_ex_to_cq(self.ffi_ptr());

            let comp_ack =
                self.comp_events_completed.load(atomic::Ordering::Relaxed) as os::raw::c_uint;
            // if the number overflows, the behavior is unspecified
            ibverbs_sys::ibv_ack_cq_events(cq, comp_ack);

            let ret = ibverbs_sys::ibv_destroy_cq(cq);
            assert_eq!(ret, 0);
        };
    }
}

#[derive(Default)]
pub struct CompletionQueueOptions {
    cqe: usize,
    user_data: usize,
    channel: Option<CompChannel>,
}

impl CompletionQueueOptions {
    #[inline]
    pub fn cqe(&mut self, cqe: usize) -> &mut Self {
        self.cqe = cqe;
        self
    }
    #[inline]
    pub fn user_data(&mut self, user_data: usize) -> &mut Self {
        self.user_data = user_data;
        self
    }
    #[inline]
    pub fn channel(&mut self, cc: &CompChannel) -> &mut Self {
        self.channel = Some(cc.clone());
        self
    }
}
