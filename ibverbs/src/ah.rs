use crate::device::Gid;
use crate::error::create_resource;
use crate::pd::ProtectionDomain;

use std::{io, mem, ptr, sync};

#[derive(Clone)]
pub struct AddressHandle(sync::Arc<Owner>);

impl AddressHandle {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_ah {
        self.0.ffi_ptr()
    }

    #[inline]
    #[must_use]
    pub fn options() -> AddressHandleOptions {
        AddressHandleOptions::default()
    }

    #[inline]
    pub fn create(pd: &ProtectionDomain, mut options: AddressHandleOptions) -> io::Result<Self> {
        // SAFETY: ffi
        let owner = unsafe {
            let attr = &mut options.attr;
            let ah = create_resource(
                || ibverbs_sys::ibv_create_ah(pd.ffi_ptr(), attr),
                || "failed to create address handle",
            )?;
            sync::Arc::new(Owner {
                ah,
                _pd: pd.clone(),
            })
        };
        Ok(Self(owner))
    }
}

struct Owner {
    ah: ptr::NonNull<ibverbs_sys::ibv_ah>,

    _pd: ProtectionDomain,
}

// SAFETY: owned type
unsafe impl Send for Owner {}
// SAFETY: owned type
unsafe impl Sync for Owner {}

impl Owner {
    fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_ah {
        self.ah.as_ptr()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe {
            let ah = self.ffi_ptr();
            let ret = ibverbs_sys::ibv_destroy_ah(ah);
            assert_eq!(ret, 0);
        }
    }
}

#[derive(Clone)]
pub struct AddressHandleOptions {
    attr: ibverbs_sys::ibv_ah_attr,
}

impl Default for AddressHandleOptions {
    #[inline]
    fn default() -> Self {
        Self {
            // SAFETY: POD ffi type
            attr: unsafe { mem::zeroed() },
        }
    }
}

impl AddressHandleOptions {
    pub(crate) fn into_ctype(self) -> ibverbs_sys::ibv_ah_attr {
        self.attr
    }

    #[inline]
    pub fn dest_lid(&mut self, dest_lid: u16) -> &mut Self {
        self.attr.dlid = dest_lid;
        self
    }

    #[inline]
    pub fn service_level(&mut self, service_level: u8) -> &mut Self {
        self.attr.sl = service_level;
        self
    }

    #[inline]
    pub fn port_num(&mut self, port_num: u8) -> &mut Self {
        self.attr.port_num = port_num;
        self
    }

    #[inline]
    pub fn global_route_header(&mut self, global_route_header: GlobalRoute) -> &mut Self {
        self.attr.is_global = 1;
        self.attr.grh = global_route_header.into_ctype();
        self
    }
}

#[repr(C)]
pub struct GlobalRoute {
    pub dest_gid: Gid,
    pub flow_label: u32,
    pub sgid_index: u8,
    pub hop_limit: u8,
    pub traffic_class: u8,
}

impl GlobalRoute {
    fn into_ctype(self) -> ibverbs_sys::ibv_global_route {
        // SAFETY: same repr
        unsafe { mem::transmute(self) }
    }
}
