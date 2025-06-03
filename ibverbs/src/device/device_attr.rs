use crate::ctx::Context;
use crate::error::from_errno;

use std::{io, mem};

pub struct DeviceAttr(ibverbs_sys::ibv_device_attr);

impl DeviceAttr {
    #[inline]
    pub fn query(ctx: &Context) -> io::Result<Self> {
        // SAFETY: ffi
        unsafe {
            let mut device_attr = mem::zeroed();
            let context = ctx.ffi_ptr();
            let ret = ibverbs_sys::ibv_query_device(context, &mut device_attr);
            if ret != 0 {
                return Err(from_errno(ret));
            }
            Ok(Self(device_attr))
        }
    }
}
