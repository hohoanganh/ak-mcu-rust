//! Phan logic de mot app Rust song chung voi bootloader cua ak-mcu-base.
//!
//! - [`boot_ctrl`]: khoi dieu khien boot <-> app trong EEPROM (hai o 32 B ghi xen ke),
//!   cung dinh dang voi `services/fw/boot_ctrl.c` cua ak-mcu-base.
//! - [`khung`]: khung giao thuc nap `fw_proto` (A5 | cmd | seq | len | payload | crc16).
//! - [`crc16`], [`crc32`]: dung nhu `common/crc.c`.
//!
//! Khong phu thuoc chip: test tren may tinh bang
//! `cargo test -p ak-boot --target x86_64-pc-windows-gnu`.

#![cfg_attr(not(test), no_std)]

pub mod boot_ctrl;
pub mod khung;

/// CRC-16/CCITT-FALSE: da thuc 0x1021, gia tri dau 0xFFFF (khung fw_proto).
pub fn crc16(du_lieu: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in du_lieu {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

/// CRC-32 chuan (IEEE, phan xa, 0xEDB88320), giong `crc32_update(0, ...)` cua ak-mcu-base.
pub fn crc32(du_lieu: &[u8]) -> u32 {
    let mut crc: u32 = !0;
    for &b in du_lieu {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_chuan() {
        assert_eq!(crc16(b"123456789"), 0x29B1);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
