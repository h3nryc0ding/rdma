use super::Guid;

use crate::ctx::Context;
use crate::error::last_error;

use std::os::raw::c_int;
use std::{ffi, fmt, io, ops, ptr, slice};

use numeric_cast::NumericCast;
use scopeguard::guard_on_unwind;

/// An array of RDMA devices.
pub struct DeviceList {
    arr: ptr::NonNull<Device>,
    len: usize,
}

/// SAFETY: owned array
unsafe impl Send for DeviceList {}
/// SAFETY: owned array
unsafe impl Sync for DeviceList {}

/// A RDMA device
#[repr(transparent)]
pub struct Device(ptr::NonNull<ibverbs_sys::ibv_device>);

/// SAFETY: owned type
unsafe impl Send for Device {}
/// SAFETY: owned type
unsafe impl Sync for Device {}

impl DeviceList {
    fn ffi_ptr(&self) -> *mut *mut ibverbs_sys::ibv_device {
        self.arr.as_ptr().cast()
    }

    /// Returns available rdma devices
    ///
    /// # Panics
    /// + if the number of devices can not be converted to an usize
    /// + if the total size of the device array is larger than slice size limit
    #[inline]
    pub fn available() -> io::Result<Self> {
        // SAFETY: ffi
        unsafe {
            let mut num_devices: c_int = 0;
            let arr = ibverbs_sys::ibv_get_device_list(&mut num_devices);
            if arr.is_null() {
                return Err(last_error());
            }

            let arr: ptr::NonNull<Device> = ptr::NonNull::new_unchecked(arr.cast());

            let _guard = guard_on_unwind((), |()| {
                ibverbs_sys::ibv_free_device_list(arr.as_ptr().cast())
            });

            let len: usize = num_devices.numeric_cast();

            if size_of::<c_int>() >= size_of::<usize>() {
                let total_size = len.saturating_mul(size_of::<*mut ibverbs_sys::ibv_device>());
                assert!(total_size < usize::MAX.wrapping_div(2));
            }

            Ok(Self { arr, len })
        }
    }

    /// Returns the slice of devices
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Device] {
        // SAFETY: guaranteed by `DeviceList::available`
        unsafe { slice::from_raw_parts(self.arr.as_ptr(), self.len) }
    }
}

impl Drop for DeviceList {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: ffi
        unsafe { ibverbs_sys::ibv_free_device_list(self.ffi_ptr()) }
    }
}

impl ops::Deref for DeviceList {
    type Target = [Device];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Device {
    pub(crate) fn ffi_ptr(&self) -> *mut ibverbs_sys::ibv_device {
        self.0.as_ptr()
    }

    /// Returns kernel device name
    #[inline]
    #[must_use]
    pub fn c_name(&self) -> &ffi::CStr {
        // SAFETY: ffi
        unsafe { ffi::CStr::from_ptr(ibverbs_sys::ibv_get_device_name(self.ffi_ptr())) }
    }

    /// Returns kernel device name
    ///
    /// # Panics
    /// + if the device name is not a valid utf8 string
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.c_name().to_str().expect("non-utf8 device name")
    }

    /// Returns device’s node GUID
    #[inline]
    #[must_use]
    pub fn guid(&self) -> Guid {
        // SAFETY: ffi
        unsafe {
            let guid = ibverbs_sys::ibv_get_device_guid(self.ffi_ptr());
            Guid::from_bytes(guid.to_ne_bytes())
        }
    }

    #[inline]
    pub fn open(&self) -> io::Result<Context> {
        Context::open(self)
    }
}
