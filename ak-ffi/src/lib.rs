//! Thu vien tinh (`libak_ffi.a`) de firmware C cua ak-mcu-base dung hai bo tach khung viet
//! bang Rust:
//! - `fw_proto` (`ak_boot::khung`), thay may trang thai C trong `fw_proto.c` — tuy chon build
//!   `AK_RUST_PROTO` cua ak-mcu-base;
//! - yeu cau Modbus RTU phia slave (`ak_modbus`), thay `nmbs_server_poll()` trong `mb_slave.c`
//!   — tuy chon `AK_RUST_MODBUS`.
//!
//! Giao dien C: `include/ak_rs.h`.
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

// ---------------------------------------------------------------- Modbus RTU (slave)

/// Ket qua tach mot khung Modbus (giong `ak_rs_mb_t` trong `ak_rs.h`).
#[repr(C)]
pub struct AkRsMb {
    /// 0 chua du byte, 1 bo qua, 2 ngoai le, 3 yeu cau
    pub loai: u8,
    pub ma_ham: u8,
    pub quang_ba: u8,
    /// Ma ngoai le khi `loai` = 2
    pub ngoai_le: u8,
    pub dia_chi: u16,
    pub so_luong: u16,
    /// 06: gia_tri[0]; 16: `so_luong` gia tri
    pub gia_tri: [u16; ak_modbus::GHI_TOI_DA as usize],
}

/// # Safety
/// `p` tro toi `n` byte doc duoc (hoac `n` = 0).
unsafe fn lat<'a>(p: *const u8, n: u16) -> &'a [u8] {
    if p.is_null() || n == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(p, n as usize)
    }
}

/// # Safety
/// `p` tro toi `n` byte ghi duoc (hoac `n` = 0).
unsafe fn lat_ghi<'a>(p: *mut u8, n: u16) -> &'a mut [u8] {
    if p.is_null() || n == 0 {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(p, n as usize)
    }
}

/// Tach mot khung yeu cau gui toi slave `dia_chi`. Tra ve `ra->loai`.
///
/// # Safety
/// `khung` tro toi `len` byte; `ra` tro toi mot `AkRsMb` ghi duoc.
#[no_mangle]
pub unsafe extern "C" fn ak_rs_mb_tach(khung: *const u8, len: u16, dia_chi: u8, ra: *mut AkRsMb) -> u8 {
    use ak_modbus::{KetQua, YeuCau};
    let ra = &mut *ra;
    ra.ma_ham = 0;
    ra.quang_ba = 0;
    ra.ngoai_le = 0;
    ra.dia_chi = 0;
    ra.so_luong = 0;
    ra.loai = match ak_modbus::tach(lat(khung, len), dia_chi) {
        KetQua::ThieuByte => 0,
        KetQua::BoQua(_) => 1,
        KetQua::NgoaiLe { ma_ham, ma, quang_ba } => {
            ra.ma_ham = ma_ham;
            ra.ngoai_le = ma;
            ra.quang_ba = quang_ba as u8;
            2
        }
        KetQua::YeuCau { yc, quang_ba } => {
            ra.ma_ham = yc.ma_ham();
            ra.quang_ba = quang_ba as u8;
            match yc {
                YeuCau::Doc { dia_chi, so_luong, .. } => {
                    ra.dia_chi = dia_chi;
                    ra.so_luong = so_luong;
                }
                YeuCau::GhiMot { dia_chi, gia_tri } => {
                    ra.dia_chi = dia_chi;
                    ra.so_luong = 1;
                    ra.gia_tri[0] = gia_tri;
                }
                YeuCau::GhiNhieu { dia_chi, so_luong, du_lieu } => {
                    ra.dia_chi = dia_chi;
                    ra.so_luong = so_luong;
                    for (i, g) in ra.gia_tri.iter_mut().take(so_luong as usize).enumerate() {
                        *g = ak_modbus::thanh_ghi(du_lieu, i);
                    }
                }
            }
            3
        }
    };
    ra.loai
}

/// Tra loi doc 03/04 voi `n` thanh ghi. Tra ve so byte (0: `n` sai hoac `cap` thieu).
///
/// # Safety
/// `thanh_ghi` tro toi `n` gia tri; `ra` tro toi `cap` byte ghi duoc.
#[no_mangle]
pub unsafe extern "C" fn ak_rs_mb_tra_loi_doc(
    dia_chi: u8,
    ma_ham: u8,
    thanh_ghi: *const u16,
    n: u16,
    ra: *mut u8,
    cap: u16,
) -> u16 {
    let regs: &[u16] = if thanh_ghi.is_null() || n == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(thanh_ghi, n as usize)
    };
    ak_modbus::tra_loi_doc(dia_chi, ma_ham, regs, lat_ghi(ra, cap)) as u16
}

/// Tra loi ghi 06/16 (lap lai `a`, `b`). Tra ve 8, hoac 0 neu `cap` < 8.
///
/// # Safety
/// `ra` tro toi `cap` byte ghi duoc.
#[no_mangle]
pub unsafe extern "C" fn ak_rs_mb_tra_loi_ghi(dia_chi: u8, ma_ham: u8, a: u16, b: u16, ra: *mut u8, cap: u16) -> u16 {
    ak_modbus::tra_loi_ghi(dia_chi, ma_ham, a, b, lat_ghi(ra, cap)) as u16
}

/// Tra loi ngoai le. Tra ve 5, hoac 0 neu `cap` < 5.
///
/// # Safety
/// `ra` tro toi `cap` byte ghi duoc.
#[no_mangle]
pub unsafe extern "C" fn ak_rs_mb_ngoai_le(dia_chi: u8, ma_ham: u8, ma: u8, ra: *mut u8, cap: u16) -> u16 {
    ak_modbus::ngoai_le(dia_chi, ma_ham, ma, lat_ghi(ra, cap)) as u16
}

extern "C" {
    /// Phia C: `FATAL("RS", 0x01)`.
    fn ak_rust_panic() -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { ak_rust_panic() }
}
