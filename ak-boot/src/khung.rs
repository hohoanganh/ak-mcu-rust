//! Khung giao thuc nap `fw_proto` cua ak-mcu-base.
//!
//! `A5 | cmd | seq | len (LE16) | du lieu[len] | crc16 (LE)`, crc16 tinh tu `cmd` toi het
//! du lieu. Tra loi: `cmd | 0x80`, cung `seq`, byte dau du lieu = ma loi (0 = OK).
//! 0xA5 khong phai ky tu ASCII nen khung di chung cong COM voi chu in ra.

use crate::crc16;

pub const SOF: u8 = 0xA5;
pub const TRA_LOI: u8 = 0x80;
pub const CMD_INFO: u8 = 0x01;
pub const CMD_LOADER: u8 = 0x06;
pub const CMD_RESET: u8 = 0x07;
pub const CMD_CONFIRM: u8 = 0x09;
pub const LOI_CMD: u8 = 11; // FW_ERR_CMD: lenh khong ho tro
pub const LOI_FLASH: u8 = 4; // FW_ERR_FLASH
/// Toi da 4 B offset + 128 B du lieu (FW_PROTO_MAX_PAYLOAD)
pub const DU_LIEU_TOI_DA: usize = 132;
/// Hai byte cach nhau qua lau thi bo khung dang do
pub const HET_GIO_MS: u32 = 500;

pub struct Khung {
    pub cmd: u8,
    pub seq: u8,
    pub du_lieu: heapless::Vec<u8, DU_LIEU_TOI_DA>,
}

#[derive(Clone, Copy, PartialEq)]
enum Buoc {
    Sof,
    Cmd,
    Seq,
    DaiThap,
    DaiCao,
    DuLieu,
    CrcThap,
    CrcCao,
}

/// Bo tach khung: cho tung byte vao `nap()`, khi du mot khung dung CRC thi nhan ra `Some`.
pub struct BoTach {
    buoc: Buoc,
    cmd: u8,
    seq: u8,
    dai: usize,
    du_lieu: heapless::Vec<u8, DU_LIEU_TOI_DA>,
    crc_thap: u8,
    lan_cuoi_ms: u32,
}

impl Default for BoTach {
    fn default() -> Self {
        Self::new()
    }
}

impl BoTach {
    pub const fn new() -> Self {
        BoTach {
            buoc: Buoc::Sof,
            cmd: 0,
            seq: 0,
            dai: 0,
            du_lieu: heapless::Vec::new(),
            crc_thap: 0,
            lan_cuoi_ms: 0,
        }
    }

    /// Dang o giua mot khung (giong `fw_proto_busy()` cua C).
    pub fn dang_trong_khung(&self) -> bool {
        self.buoc != Buoc::Sof
    }

    /// Byte `b` den luc `bay_gio_ms` co thuoc giao thuc khong — goi TRUOC `nap()`.
    /// Giong gia tri tra ve cua `fw_proto_feed()` ban C: dang giua khung (chua het gio)
    /// thi moi byte deu thuoc khung, ke ca byte CRC sai hay byte do dai qua lon; con lai
    /// chi byte A5 mo dau mot khung moi. Byte khong thuoc khung la cua shell.
    pub fn byte_thuoc_khung(&self, b: u8, bay_gio_ms: u32) -> bool {
        let het_gio = bay_gio_ms.wrapping_sub(self.lan_cuoi_ms) > HET_GIO_MS;
        (self.buoc != Buoc::Sof && !het_gio) || b == SOF
    }

    pub fn nap(&mut self, b: u8, bay_gio_ms: u32) -> Option<Khung> {
        if self.buoc != Buoc::Sof && bay_gio_ms.wrapping_sub(self.lan_cuoi_ms) > HET_GIO_MS {
            self.buoc = Buoc::Sof; // khung bi ngat giua chung
        }
        self.lan_cuoi_ms = bay_gio_ms;
        match self.buoc {
            Buoc::Sof => {
                if b == SOF {
                    self.buoc = Buoc::Cmd;
                }
            }
            Buoc::Cmd => {
                self.cmd = b;
                self.buoc = Buoc::Seq;
            }
            Buoc::Seq => {
                self.seq = b;
                self.buoc = Buoc::DaiThap;
            }
            Buoc::DaiThap => {
                self.dai = b as usize;
                self.buoc = Buoc::DaiCao;
            }
            Buoc::DaiCao => {
                self.dai |= (b as usize) << 8;
                self.du_lieu.clear();
                self.buoc = if self.dai > DU_LIEU_TOI_DA {
                    Buoc::Sof // khung qua dai: bo
                } else if self.dai == 0 {
                    Buoc::CrcThap
                } else {
                    Buoc::DuLieu
                };
            }
            Buoc::DuLieu => {
                self.du_lieu.push(b).ok();
                if self.du_lieu.len() == self.dai {
                    self.buoc = Buoc::CrcThap;
                }
            }
            Buoc::CrcThap => {
                self.crc_thap = b;
                self.buoc = Buoc::CrcCao;
            }
            Buoc::CrcCao => {
                self.buoc = Buoc::Sof;
                let nhan = u16::from_le_bytes([self.crc_thap, b]);
                if nhan == self.crc_tinh() {
                    return Some(Khung { cmd: self.cmd, seq: self.seq, du_lieu: self.du_lieu.clone() });
                }
            }
        }
        None
    }

    fn crc_tinh(&self) -> u16 {
        let mut dau = heapless::Vec::<u8, { 4 + DU_LIEU_TOI_DA }>::new();
        dau.extend_from_slice(&[self.cmd, self.seq, self.dai as u8, (self.dai >> 8) as u8]).ok();
        dau.extend_from_slice(&self.du_lieu).ok();
        crc16(&dau)
    }
}

/// Dong goi mot khung vao `ra`, tra ve so byte. `ra` phai du `7 + du_lieu.len()`.
pub fn dong_goi(cmd: u8, seq: u8, du_lieu: &[u8], ra: &mut [u8]) -> usize {
    let n = du_lieu.len();
    ra[0] = SOF;
    ra[1] = cmd;
    ra[2] = seq;
    ra[3] = n as u8;
    ra[4] = (n >> 8) as u8;
    ra[5..5 + n].copy_from_slice(du_lieu);
    let crc = crc16(&ra[1..5 + n]);
    ra[5 + n] = crc as u8;
    ra[6 + n] = (crc >> 8) as u8;
    7 + n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tach_het(bo: &mut BoTach, byte: &[u8], t0: u32) -> Option<Khung> {
        let mut kq = None;
        for (i, &b) in byte.iter().enumerate() {
            if let Some(k) = bo.nap(b, t0 + i as u32) {
                kq = Some(k);
            }
        }
        kq
    }

    #[test]
    fn dong_goi_roi_tach_lai() {
        let mut ra = [0u8; 64];
        let n = dong_goi(CMD_CONFIRM, 7, &[1, 2, 3], &mut ra);
        let k = tach_het(&mut BoTach::new(), &ra[..n], 0).unwrap();
        assert_eq!((k.cmd, k.seq, &k.du_lieu[..]), (CMD_CONFIRM, 7, &[1u8, 2, 3][..]));
    }

    #[test]
    fn khung_info_giong_ak_fw_py() {
        // ak_fw.py pack(CMD_INFO, 1): A5 01 01 00 00 + crc16(01 01 00 00) LE
        let crc = crc16(&[1, 1, 0, 0]);
        let byte = [SOF, 1, 1, 0, 0, crc as u8, (crc >> 8) as u8];
        let k = tach_het(&mut BoTach::new(), &byte, 0).unwrap();
        assert_eq!((k.cmd, k.seq, k.du_lieu.len()), (CMD_INFO, 1, 0));
    }

    #[test]
    fn chu_xen_giua_khung_bi_bo_qua() {
        let mut ra = [0u8; 16];
        let n = dong_goi(CMD_INFO, 3, &[], &mut ra);
        let mut byte = b"help\r\n".to_vec();
        byte.extend_from_slice(&ra[..n]);
        assert_eq!(tach_het(&mut BoTach::new(), &byte, 0).unwrap().seq, 3);
    }

    #[test]
    fn byte_ngoai_khung_de_cho_shell() {
        // Nhu fw_proto_feed() cua C: byte truoc khi het gio thi thuoc khung; het gio roi thi
        // byte khong phai A5 la cua shell.
        let mut bo = BoTach::new();
        assert!(!bo.dang_trong_khung());
        bo.nap(b'h', 0);
        assert!(!bo.dang_trong_khung()); // chu thuong: shell
        bo.nap(SOF, 1);
        assert!(bo.dang_trong_khung());
        bo.nap(b'x', 700); // het gio giua khung: bo khung, 'x' la cua shell
        assert!(!bo.dang_trong_khung());
    }

    #[test]
    fn byte_thuoc_khung_nhu_fw_proto_feed() {
        let mut ra = [0u8; 16];
        let n = dong_goi(CMD_INFO, 1, &[], &mut ra);
        ra[n - 1] ^= 0xFF; // CRC sai
        let mut bo = BoTach::new();
        for (i, &b) in ra[..n].iter().enumerate() {
            assert!(bo.byte_thuoc_khung(b, i as u32), "byte {} cua khung CRC sai", i);
            assert!(bo.nap(b, i as u32).is_none());
        }
        // khung khai do dai qua lon: ca byte do dai cao van thuoc khung
        let qua_dai = [SOF, 3, 1, 0xFF, 0x7F];
        for (i, &b) in qua_dai.iter().enumerate() {
            assert!(bo.byte_thuoc_khung(b, 10 + i as u32));
            bo.nap(b, 10 + i as u32);
        }
        assert!(!bo.dang_trong_khung());
        assert!(!bo.byte_thuoc_khung(b'h', 20)); // sau do: chu thuong cua shell
        bo.nap(SOF, 21);
        assert!(!bo.byte_thuoc_khung(b'x', 700)); // het gio giua khung: cua shell
    }

    #[test]
    fn sai_crc_thi_khong_nhan() {
        let mut ra = [0u8; 16];
        let n = dong_goi(CMD_INFO, 3, &[], &mut ra);
        ra[n - 1] ^= 0xFF;
        assert!(tach_het(&mut BoTach::new(), &ra[..n], 0).is_none());
    }

    #[test]
    fn ngat_giua_khung_qua_500_ms_thi_bat_dau_lai() {
        let mut ra = [0u8; 16];
        let n = dong_goi(CMD_INFO, 9, &[], &mut ra);
        let mut bo = BoTach::new();
        assert!(bo.nap(ra[0], 0).is_none());
        assert!(bo.nap(ra[1], 1).is_none());
        // 600 ms sau moi toi phan con lai: khung cu bi bo, khung moi ngay sau van tach duoc
        assert!(tach_het(&mut bo, &ra[2..n], 600).is_none());
        assert_eq!(tach_het(&mut bo, &ra[..n], 700).unwrap().seq, 9);
    }
}
