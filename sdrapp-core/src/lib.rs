mod device;
mod fft;
mod demod;
mod audio;
mod pipeline;

pub use pipeline::SdrappCore;
pub use fft::FFT_SIZE;
pub use demod::DemodMode;
use crate::device::{DeviceInfo, GainElementInfo};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn sdrapp_create() -> *mut SdrappCore {
    Box::into_raw(Box::new(SdrappCore::new()))
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_destroy(ptr: *mut SdrappCore) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_device(ptr: *mut SdrappCore, args: *const c_char) {
    if ptr.is_null() || args.is_null() { return; }
    let args_str = CStr::from_ptr(args).to_string_lossy();
    (*ptr).set_device(&args_str);
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_frequency(ptr: *mut SdrappCore, hz: u64) {
    if !ptr.is_null() { (*ptr).set_frequency(hz); }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_gain(ptr: *mut SdrappCore, db: f64) {
    if !ptr.is_null() { (*ptr).set_gain(db); }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_demod(ptr: *mut SdrappCore, mode: u32) {
    if ptr.is_null() { return; }
    let m = if mode == 0 { DemodMode::Am } else { DemodMode::Wbfm };
    (*ptr).set_demod(m);
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_start(ptr: *mut SdrappCore) -> bool {
    if ptr.is_null() { return false; }
    (*ptr).start()
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_stop(ptr: *mut SdrappCore) {
    if !ptr.is_null() { (*ptr).stop(); }
}

/// Kopiert FFT-Daten in out_buf. Gibt Anzahl geschriebener Werte zurück.
/// out_len muss >= 1024 sein.
#[no_mangle]
pub unsafe extern "C" fn sdrapp_get_fft(
    ptr: *const SdrappCore,
    out_buf: *mut f32,
    out_len: usize,
) -> usize {
    if ptr.is_null() || out_buf.is_null() { return 0; }
    let buf = std::slice::from_raw_parts_mut(out_buf, out_len);
    (*ptr).get_fft(buf)
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_gain_element(
    ptr: *mut SdrappCore,
    name: *const c_char,
    db: f64,
) {
    if ptr.is_null() || name.is_null() { return; }
    let name_str = CStr::from_ptr(name).to_string_lossy();
    (*ptr).set_gain_element(&name_str, db);
}

#[repr(C)]
pub struct GainElementC {
    pub name: *mut c_char,
    pub min_db: f64,
    pub max_db: f64,
    pub step_db: f64,
    pub current_db: f64,
}

#[repr(C)]
pub struct GainListC {
    pub count: usize,
    pub items: *mut GainElementC,
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_list_gains(
    ptr: *const SdrappCore,
    out_count: *mut usize,
) -> *mut GainListC {
    if ptr.is_null() { return std::ptr::null_mut(); }
    let elements: Vec<GainElementInfo> = (*ptr).list_gain_elements();
    let count = elements.len();
    if !out_count.is_null() { *out_count = count; }

    let mut items: Vec<GainElementC> = elements.into_iter().map(|e| GainElementC {
        name: CString::new(e.name).unwrap_or_default().into_raw(),
        min_db: e.min_db,
        max_db: e.max_db,
        step_db: e.step_db,
        current_db: e.current_db,
    }).collect();
    items.shrink_to_fit();
    let items_ptr = items.as_mut_ptr();
    std::mem::forget(items);

    Box::into_raw(Box::new(GainListC { count, items: items_ptr }))
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_free_gain_list(ptr: *mut GainListC) {
    if ptr.is_null() { return; }
    let list = Box::from_raw(ptr);
    if !list.items.is_null() {
        // Safety: shrink_to_fit() vor forget() garantiert capacity == len
        let items = Vec::from_raw_parts(list.items, list.count, list.count);
        for item in &items {
            if !item.name.is_null() { drop(CString::from_raw(item.name)); }
        }
    }
}

/// Gibt Anzahl angeschlossener Geräte zurück.
#[no_mangle]
pub unsafe extern "C" fn sdrapp_list_devices(
    out_count: *mut usize,
) -> *mut DeviceListC {
    let devices = SdrappCore::list_devices();
    let count = devices.len();
    if !out_count.is_null() { *out_count = count; }

    let mut items: Vec<DeviceItemC> = devices.into_iter().map(|d| DeviceItemC {
        label: CString::new(d.label).unwrap_or_default().into_raw(),
        args:  CString::new(d.args).unwrap_or_default().into_raw(),
    }).collect();
    items.shrink_to_fit();
    let items_ptr = items.as_mut_ptr();
    std::mem::forget(items); // Ownership goes to DeviceListC

    let list = Box::new(DeviceListC { count, items: items_ptr });
    Box::into_raw(list)
}

#[repr(C)]
pub struct DeviceItemC {
    pub label: *mut c_char,
    pub args: *mut c_char,
}

#[repr(C)]
pub struct DeviceListC {
    pub count: usize,
    pub items: *mut DeviceItemC,  // raw pointer — C-ABI safe
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_free_device_list(ptr: *mut DeviceListC) {
    if ptr.is_null() { return; }
    let list = Box::from_raw(ptr);
    if !list.items.is_null() {
        // Safety: shrink_to_fit() vor forget() garantiert capacity == len
        let items = Vec::from_raw_parts(list.items, list.count, list.count);
        for item in &items {
            if !item.label.is_null() { drop(CString::from_raw(item.label)); }
            if !item.args.is_null()  { drop(CString::from_raw(item.args));  }
        }
    }
}
