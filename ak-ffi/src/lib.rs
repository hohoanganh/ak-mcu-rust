//! Thu vien tinh (`libak_ffi.a`) de firmware C cua ak-mcu-base dung bo tach khung
//! `fw_proto` viet bang Rust (`ak_boot::khung`), thay may trang thai C trong `fw_proto.c`.
//!
//! Giao dien C: `include/ak_rs.h`. Bat trong ak-mcu-base bang tuy chon build `AK_RUST_PROTO`.
//! Panic cua Rust goi `ak_rust_panic()` cua phia C -> `FATAL("RS", ...)` -> nhat ky su co.

#![no_std]

use ak_boot::khung::{BoTach, DU_LIEU_TOI_DA};
use core::ptr::addr_of_mut;

/// Mot khung nhan du (giong `ak_rs_khung_t` trong `ak_rs.h`).
#[repr(C)]
pub struct AkRsKhung {
    pub cmd: u8,
    pub seq: u8,
    pub len: u16,
    pub du_lieu: [u8; DU_LIEU_TOI_DA],
}

/// Mot bo tach duy nhat, nhu `static rx_state` cua ban C. Chi goi tu mot ngu canh
/// (task console cua kernel AK, khong phai ngat) nen khong can khoa.
static mut BO_TACH: BoTach = BoTach::new();

fn bo_tach() -> &'static mut BoTach {
    unsafe { &mut *addr_of_mut!(BO_TACH) }
}

/// Bo khung dang do (`fw_proto_init`).
#[no_mangle]
pub extern "C" fn ak_rs_proto_reset() {
    *bo_tach() = BoTach::new();
}

/// 1 neu dang o giua mot khung (`fw_proto_busy`).
#[no_mangle]
pub extern "C" fn ak_rs_proto_busy() -> u8 {
    bo_tach().dang_trong_khung() as u8
}

/// Nap mot byte. Tra ve 0: byte khong thuoc khung (de cho shell); 1: byte thuoc khung;
/// 2: byte thuoc khung va vua nhan du mot khung dung CRC, da chep vao `*ra`.
///
/// # Safety
/// `ra` tro toi mot `AkRsKhung` hop le, ghi duoc.
#[no_mangle]
pub unsafe extern "C" fn ak_rs_proto_feed(b: u8, bay_gio_ms: u32, ra: *mut AkRsKhung) -> u8 {
    let bo = bo_tach();
    let thuoc_khung = bo.byte_thuoc_khung(b, bay_gio_ms);
    if let Some(k) = bo.nap(b, bay_gio_ms) {
        let ra = &mut *ra;
        ra.cmd = k.cmd;
        ra.seq = k.seq;
        ra.len = k.du_lieu.len() as u16;
        ra.du_lieu[..k.du_lieu.len()].copy_from_slice(&k.du_lieu);
        return 2;
    }
    thuoc_khung as u8
}

extern "C" {
    /// Phia C: `FATAL("RS", 0x01)`.
    fn ak_rust_panic() -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { ak_rust_panic() }
}
