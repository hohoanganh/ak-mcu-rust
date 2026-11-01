//! Nhat ky su co trong NVM — cung dinh dang voi `services/sys/crash_log.c` cua ak-mcu-base,
//! de app C doc duoc su co cua bai Rust (lenh `crash`) va nguoc lai.
//!
//! Vong 8 ban ghi x 20 B tu offset 64 (ngay sau `boot_ctrl`). Ban ghi moi nhat la ban co
//! `seq` lon nhat (so sanh vong qua 255 nhu C); ban ghi bi mat dien cat ngang sai CRC va
//! bi bo qua. Ban ghi: seq loai ma task | pc(4) | lr(4) | info(4) | sig reserved | crc16(2),
//! so nguyen little-endian, crc16 (CCITT, nhu `fw_proto`) tinh tren 18 byte dau.

use crate::boot_ctrl::{LoiNvm, Nvm};
use crate::crc16;

pub const VI_TRI: u32 = 64;
pub const SO_O: u8 = 8;
const KICH_THUOC: usize = 20;

pub const LOAI_HARDFAULT: u8 = 1;
pub const LOAI_FATAL: u8 = 2;
pub const LOAI_WATCHDOG: u8 = 3;
pub const LOAI_TASK_STALLED: u8 = 4;

/// `task` khi khong co task nao dang chay (`AK_TASK_IDLE_ID` cua kernel AK). Bai Rust
/// khong co ma task cua kernel AK nen luon dung so nay.
pub const TASK_IDLE: u8 = 0xEF;

/// `info` cua ban ghi FATAL do panic cua bai Rust: the "RUST". Khi do `pc` = so dong,
/// `lr` = 4 ky tu dau cua ten tep (vi du "main").
pub const THE_RUST: u32 = u32::from_le_bytes(*b"RUST");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BanGhi {
    /// Do `them()` dat
    pub seq: u8,
    /// `LOAI_*`
    pub loai: u8,
    /// Ma cua FATAL / so task bi bo doi
    pub ma: u8,
    pub task: u8,
    pub pc: u32,
    pub lr: u32,
    /// HARDFAULT: CFSR. FATAL: 4 ky tu dau cua the
    pub info: u32,
    pub sig: u8,
}

fn u32_le(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

impl BanGhi {
    pub fn sang_byte(&self) -> [u8; KICH_THUOC] {
        let mut b = [0u8; KICH_THUOC];
        b[0] = self.seq;
        b[1] = self.loai;
        b[2] = self.ma;
        b[3] = self.task;
        b[4..8].copy_from_slice(&self.pc.to_le_bytes());
        b[8..12].copy_from_slice(&self.lr.to_le_bytes());
        b[12..16].copy_from_slice(&self.info.to_le_bytes());
        b[16] = self.sig;
        b[17] = 0;
        let crc = crc16(&b[..18]);
        b[18..20].copy_from_slice(&crc.to_le_bytes());
        b
    }

    /// None neu o trong (`loai` 0) hoac sai CRC.
    pub fn tu_byte(b: &[u8; KICH_THUOC]) -> Option<Self> {
        if b[1] == 0 || u16::from_le_bytes([b[18], b[19]]) != crc16(&b[..18]) {
            return None;
        }
        Some(BanGhi {
            seq: b[0],
            loai: b[1],
            ma: b[2],
            task: b[3],
            pc: u32_le(b, 4),
            lr: u32_le(b, 8),
            info: u32_le(b, 12),
            sig: b[16],
        })
    }
}

fn doc_o(nvm: &mut impl Nvm, o: u8) -> Option<BanGhi> {
    let mut b = [0u8; KICH_THUOC];
    nvm.doc(VI_TRI + o as u32 * KICH_THUOC as u32, &mut b).ok()?;
    BanGhi::tu_byte(&b)
}

/// O chua ban ghi moi nhat va `seq` cua no.
fn moi_nhat(nvm: &mut impl Nvm) -> Option<(u8, u8)> {
    let mut tot: Option<(u8, u8)> = None;
    for o in 0..SO_O {
        if let Some(r) = doc_o(nvm, o) {
            match tot {
                Some((_, seq)) if (r.seq.wrapping_sub(seq) as i8) <= 0 => {}
                _ => tot = Some((o, r.seq)),
            }
        }
    }
    tot
}

/// Them mot ban ghi (dat `seq`), ghi vao o sau o moi nhat, roi doc lai de kiem.
pub fn them(nvm: &mut impl Nvm, r: &mut BanGhi) -> Result<(), LoiNvm> {
    let o = match moi_nhat(nvm) {
        None => {
            r.seq = 1;
            0
        }
        Some((o, seq)) => {
            r.seq = seq.wrapping_add(1);
            (o + 1) % SO_O
        }
    };
    let b = r.sang_byte();
    let vi_tri = VI_TRI + o as u32 * KICH_THUOC as u32;
    nvm.ghi(vi_tri, &b)?;
    let mut lai = [0u8; KICH_THUOC];
    nvm.doc(vi_tri, &mut lai)?;
    if lai != b {
        return Err(LoiNvm::KiemLai);
    }
    Ok(())
}

/// Ban ghi moi thu `n` (0 = moi nhat). Dung o cho dut dau tien trong day `seq`.
pub fn doc(nvm: &mut impl Nvm, n: u8) -> Option<BanGhi> {
    let (o, seq) = moi_nhat(nvm)?;
    if n >= SO_O {
        return None;
    }
    let r = doc_o(nvm, (o + SO_O - n) % SO_O)?;
    (r.seq == seq.wrapping_sub(n)).then_some(r)
}

/// So ban ghi hop le, lien mach tu ban moi nhat.
pub fn dem(nvm: &mut impl Nvm) -> u8 {
    (0..SO_O).take_while(|&n| doc(nvm, n).is_some()).count() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NvmGia([u8; 256]);

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

    /// Ban ghi thu i nhu `golden.c` (xem tests/crash_log_c.bin) da ghi.
    fn mau(i: u32) -> BanGhi {
        BanGhi {
            seq: 0,
            loai: (1 + i % 4) as u8,
            ma: (0x10 + i) as u8,
            task: if i == 10 { TASK_IDLE } else { i as u8 },
            pc: 0x0800_3100 + i * 0x111,
            lr: 0x0800_FFF0 + i,
            info: if i.is_multiple_of(4) { 0x8200 + i } else { THE_RUST },
            sig: (10 + i) as u8,
        }
    }

    fn anh_c() -> NvmGia {
        let b = include_bytes!("../tests/crash_log_c.bin");
        let mut n = [0xFFu8; 256];
        n[..b.len()].copy_from_slice(b);
        NvmGia(n)
    }

    #[test]
    fn doc_duoc_nhat_ky_do_ban_c_ghi() {
        // crash_log.c ghi 10 ban ghi vao 8 o: con 8 ban moi nhat (3..10), seq 3..10
        let mut nvm = anh_c();
        assert_eq!(dem(&mut nvm), 8);
        for n in 0..8u8 {
            let i = 10 - n as u32;
            let mut ky_vong = mau(i);
            ky_vong.seq = i as u8;
            assert_eq!(doc(&mut nvm, n), Some(ky_vong), "ban ghi {}", n);
        }
        assert_eq!(doc(&mut nvm, 8), None);
    }

    #[test]
    fn ghi_ra_dung_tung_byte_nhu_ban_c() {
        let mut nvm = NvmGia([0xFF; 256]);
        for i in 1..=10 {
            them(&mut nvm, &mut mau(i)).unwrap();
        }
        assert_eq!(&nvm.0[..64 + 160], &anh_c().0[..64 + 160]);
    }

    #[test]
    fn nvm_trong_va_ban_ghi_bi_cat() {
        let mut nvm = NvmGia([0xFF; 256]);
        assert_eq!((dem(&mut nvm), doc(&mut nvm, 0)), (0, None));
        them(&mut nvm, &mut mau(1)).unwrap();
        them(&mut nvm, &mut mau(2)).unwrap();
        assert_eq!(dem(&mut nvm), 2);
        // mat dien giua luc ghi ban thu 3: o do sai CRC, hai ban cu con nguyen
        let mut r = mau(3);
        r.seq = 3;
        let b = r.sang_byte();
        let o = (VI_TRI + 2 * 20) as usize;
        nvm.0[o..o + 11].copy_from_slice(&b[..11]);
        assert_eq!(dem(&mut nvm), 2);
        assert_eq!(doc(&mut nvm, 0).unwrap().seq, 2);
    }

    #[test]
    fn seq_vong_qua_255() {
        let mut nvm = NvmGia([0xFF; 256]);
        for i in 0..300u32 {
            them(&mut nvm, &mut mau(1 + i % 9)).unwrap();
        }
        assert_eq!(dem(&mut nvm), 8);
        assert_eq!(doc(&mut nvm, 0).unwrap().seq, (300 % 256) as u8);
        assert_eq!(doc(&mut nvm, 7).unwrap().seq, (293 % 256) as u8);
    }
}
