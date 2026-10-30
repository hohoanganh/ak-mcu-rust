//! Khoi dieu khien boot <-> app (`boot_ctrl_t` cua ak-mcu-base), trong NVM tu offset 0.
//!
//! 20 B: magic(4) cmd attempts last_result seq install_count(4) trial trial_boots
//! rolled_back reserved crc32(4). Hai o 32 B ghi xen ke: mat dien giua luc ghi chi
//! hong o dang ghi, o kia van giu trang thai truoc. O moi nhat = `seq` lon hon
//! (so sanh vong qua 255 nhu C: `(i8)(seq1 - seq0) > 0`).

use crate::crc32;

pub const MAGIC: u32 = 0x4C54_434B; // "KCTL"
pub const CMD_NONE: u8 = 0;
pub const CMD_UPDATE: u8 = 1;
pub const CMD_LOADER: u8 = 2;
pub const CMD_ROLLBACK: u8 = 3;

const KICH_THUOC: usize = 20;
const O_NHO: u32 = 32;

/// Doc / ghi vung NVM (EEPROM) theo offset tinh tu dau vung.
pub trait Nvm {
    fn doc(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), LoiNvm>;
    fn ghi(&mut self, offset: u32, du_lieu: &[u8]) -> Result<(), LoiNvm>;
}

#[derive(Debug, PartialEq)]
pub enum LoiNvm {
    Doc,
    Ghi,
    /// Ghi xong doc lai khong khop
    KiemLai,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BootCtrl {
    pub cmd: u8,
    pub install_attempts: u8,
    pub last_result: u8,
    pub seq: u8,
    pub install_count: u32,
    /// 1: anh dang chay vua cai, chua tu xac nhan
    pub trial: u8,
    pub trial_boots: u8,
    pub rolled_back: u8,
    pub reserved: u8,
}

impl BootCtrl {
    /// Giai ma mot o; None neu sai magic hoac sai CRC.
    pub fn tu_byte(b: &[u8; KICH_THUOC]) -> Option<Self> {
        let magic = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        let crc = u32::from_le_bytes([b[16], b[17], b[18], b[19]]);
        if magic != MAGIC || crc != crc32(&b[..16]) {
            return None;
        }
        Some(BootCtrl {
            cmd: b[4],
            install_attempts: b[5],
            last_result: b[6],
            seq: b[7],
            install_count: u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
            trial: b[12],
            trial_boots: b[13],
            rolled_back: b[14],
            reserved: b[15],
        })
    }

    /// Ma hoa kem magic va CRC.
    pub fn sang_byte(&self) -> [u8; KICH_THUOC] {
        let mut b = [0u8; KICH_THUOC];
        b[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        b[4] = self.cmd;
        b[5] = self.install_attempts;
        b[6] = self.last_result;
        b[7] = self.seq;
        b[8..12].copy_from_slice(&self.install_count.to_le_bytes());
        b[12] = self.trial;
        b[13] = self.trial_boots;
        b[14] = self.rolled_back;
        b[15] = self.reserved;
        let crc = crc32(&b[..16]);
        b[16..20].copy_from_slice(&crc.to_le_bytes());
        b
    }
}

fn doc_o(nvm: &mut impl Nvm, o: u32) -> Option<BootCtrl> {
    let mut b = [0u8; KICH_THUOC];
    nvm.doc(o * O_NHO, &mut b).ok()?;
    BootCtrl::tu_byte(&b)
}

/// O moi nhat con hop le va noi dung cua no.
fn moi_nhat(nvm: &mut impl Nvm) -> Option<(u32, BootCtrl)> {
    match (doc_o(nvm, 0), doc_o(nvm, 1)) {
        (Some(a), Some(b)) => {
            if (b.seq.wrapping_sub(a.seq) as i8) > 0 {
                Some((1, b))
            } else {
                Some((0, a))
            }
        }
        (Some(a), None) => Some((0, a)),
        (None, Some(b)) => Some((1, b)),
        (None, None) => None,
    }
}

/// Trang thai hien tai; NVM trong / hong thi tra mac dinh (cmd NONE).
pub fn doc(nvm: &mut impl Nvm) -> BootCtrl {
    moi_nhat(nvm).map(|(_, c)| c).unwrap_or_default()
}

/// Ghi vao o KHONG chua trang thai moi nhat, tang `seq`, roi doc lai de kiem.
pub fn luu(nvm: &mut impl Nvm, ctrl: &mut BootCtrl) -> Result<(), LoiNvm> {
    let o = match moi_nhat(nvm) {
        None => {
            ctrl.seq = 1;
            0
        }
        Some((o, cu)) => {
            ctrl.seq = cu.seq.wrapping_add(1);
            o ^ 1
        }
    };
    let b = ctrl.sang_byte();
    nvm.ghi(o * O_NHO, &b)?;
    let mut lai = [0u8; KICH_THUOC];
    nvm.doc(o * O_NHO, &mut lai)?;
    if lai != b {
        return Err(LoiNvm::KiemLai);
    }
    Ok(())
}

/// Dat lenh cho bootloader (vi du `CMD_LOADER`). Khong ghi neu lenh da dung.
pub fn dat_lenh(nvm: &mut impl Nvm, cmd: u8) -> Result<(), LoiNvm> {
    let mut c = doc(nvm);
    if c.cmd == cmd {
        return Ok(());
    }
    c.cmd = cmd;
    luu(nvm, &mut c)
}

/// Anh dang chay da on: ket thuc che do chay thu. Tra ve true neu no dang chay thu.
pub fn xac_nhan(nvm: &mut impl Nvm) -> Result<bool, LoiNvm> {
    let mut c = doc(nvm);
    if c.trial == 0 {
        return Ok(false);
    }
    c.trial = 0;
    c.trial_boots = 0;
    luu(nvm, &mut c)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NVM gia trong RAM, co the "lam hong" mot o.
    struct NvmGia([u8; 64]);

    impl Nvm for NvmGia {
        fn doc(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), LoiNvm> {
            let o = offset as usize;
            buf.copy_from_slice(&self.0[o..o + buf.len()]);
            Ok(())
        }
        fn ghi(&mut self, offset: u32, du_lieu: &[u8]) -> Result<(), LoiNvm> {
            let o = offset as usize;
            self.0[o..o + du_lieu.len()].copy_from_slice(du_lieu);
            Ok(())
        }
    }

    fn eeprom_kit() -> NvmGia {
        let b = include_bytes!("../tests/eeprom_kit_20261030.bin");
        let mut n = [0u8; 64];
        n.copy_from_slice(&b[..64]);
        NvmGia(n)
    }

    #[test]
    fn doc_eeprom_that_cua_kit() {
        // Dump 30/10/2026: o 0 seq 65 (trial 1), o 1 seq 66 (da xac nhan) -> o 1 moi nhat
        let mut nvm = eeprom_kit();
        let c = doc(&mut nvm);
        assert_eq!(c.seq, 66);
        assert_eq!(c.install_count, 9);
        assert_eq!(c.trial, 0);
        assert_eq!(c.cmd, CMD_NONE);
    }

    #[test]
    fn nvm_trong_tra_mac_dinh_va_ghi_o_0() {
        let mut nvm = NvmGia([0xFF; 64]);
        assert_eq!(doc(&mut nvm), BootCtrl::default());
        dat_lenh(&mut nvm, CMD_LOADER).unwrap();
        let c = doc(&mut nvm);
        assert_eq!((c.cmd, c.seq), (CMD_LOADER, 1));
        assert!(doc_o(&mut nvm, 0).is_some() && doc_o(&mut nvm, 1).is_none());
    }

    #[test]
    fn luu_xen_ke_hai_o_khong_ghi_de_o_moi_nhat() {
        let mut nvm = eeprom_kit(); // moi nhat o 1
        dat_lenh(&mut nvm, CMD_LOADER).unwrap();
        assert_eq!(doc_o(&mut nvm, 0).unwrap().seq, 67); // ghi vao o 0
        assert_eq!(doc_o(&mut nvm, 1).unwrap().seq, 66); // o 1 con nguyen
        dat_lenh(&mut nvm, CMD_NONE).unwrap();
        assert_eq!(doc_o(&mut nvm, 1).unwrap().seq, 68);
        assert_eq!(doc(&mut nvm).cmd, CMD_NONE);
    }

    #[test]
    fn seq_vong_qua_255() {
        let mut nvm = NvmGia([0xFF; 64]);
        let mut a = BootCtrl { seq: 255, ..Default::default() };
        let mut b = BootCtrl { seq: 0, cmd: CMD_LOADER, ..Default::default() };
        nvm.ghi(0, &a.sang_byte()).unwrap();
        nvm.ghi(32, &b.sang_byte()).unwrap();
        assert_eq!(doc(&mut nvm).cmd, CMD_LOADER); // 0 moi hon 255
        a.seq = 1;
        b.seq = 0;
        nvm.ghi(0, &a.sang_byte()).unwrap();
        nvm.ghi(32, &b.sang_byte()).unwrap();
        assert_eq!(doc(&mut nvm).cmd, CMD_NONE);
    }

    #[test]
    fn mat_dien_giua_luc_ghi_van_con_trang_thai_cu() {
        let mut nvm = eeprom_kit();
        nvm.0[32 + 9] ^= 0x01; // o 1 (moi nhat) bi hong
        let c = doc(&mut nvm);
        assert_eq!((c.seq, c.trial), (65, 1)); // lui ve o 0
    }

    #[test]
    fn xac_nhan_ket_thuc_chay_thu() {
        let mut nvm = NvmGia([0xFF; 64]);
        let mut c = BootCtrl { trial: 1, trial_boots: 2, install_count: 3, ..Default::default() };
        luu(&mut nvm, &mut c).unwrap();
        assert_eq!(xac_nhan(&mut nvm), Ok(true));
        let c = doc(&mut nvm);
        assert_eq!((c.trial, c.trial_boots, c.install_count), (0, 0, 3));
        assert_eq!(xac_nhan(&mut nvm), Ok(false)); // khong con gi de xac nhan
    }
}
