//! Tach khung yeu cau Modbus RTU phia slave, va dung cau tra loi.
//!
//! Slave chi goi [`tach`] khi duong day da im 3,5 ky tu: luc do ca khung yeu cau da nam
//! trong bo dem. `tach` kiem dia chi, ma ham, do dai, CRC va gioi han so thanh ghi, roi
//! tra ve mot trong bon ket qua: chua du byte, bo qua (khong tra loi), ngoai le (tra
//! loi ma loi), hoac mot yeu cau hop le de app doc/ghi thanh ghi.
//!
//! Ma ham ho tro: 03 doc holding, 04 doc input, 06 ghi mot, 16 ghi nhieu — dung bon ham
//! slave cua ak-mcu-base. Thu tu kiem va ma ngoai le giong nanoMODBUS (ma C ak-mcu-base
//! dang dung), khac o hai cho, deu theo dac ta Modbus:
//! - khung dai hon do dai cua ma ham thi bo ca khung (nanoMODBUS tra loi phan dau);
//! - ma ham khong ho tro: chi tra ngoai le 01 khi CRC dung (nanoMODBUS tra ca khi CRC sai).
//!
//! Khong phu thuoc chip, khong cap phat: test tren may tinh bang
//! `cargo test -p ak-modbus --target <target cua may>`.

#![cfg_attr(not(test), no_std)]

/// Dia chi quang ba: moi slave lam theo, khong ai tra loi.
pub const QUANG_BA: u8 = 0;

pub const DOC_HOLDING: u8 = 3;
pub const DOC_INPUT: u8 = 4;
pub const GHI_MOT: u8 = 6;
pub const GHI_NHIEU: u8 = 16;

/// Ma ngoai le Modbus.
pub const NL_MA_HAM: u8 = 1;
pub const NL_DIA_CHI: u8 = 2;
pub const NL_GIA_TRI: u8 = 3;
pub const NL_THIET_BI: u8 = 4;

/// So thanh ghi toi da mot lan doc / mot lan ghi nhieu (dac ta Modbus).
pub const DOC_TOI_DA: u16 = 125;
pub const GHI_TOI_DA: u16 = 123;

/// Khung RTU dai nhat theo dac ta.
pub const KHUNG_TOI_DA: usize = 256;

/// CRC-16/MODBUS: da thuc 0xA001 (phan xa), gia tri dau 0xFFFF. Tren day: byte thap truoc.
pub fn crc(du_lieu: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in du_lieu {
        crc ^= b as u16;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        }
    }
    crc
}

/// Mot yeu cau hop le.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YeuCau<'a> {
    /// Ma ham 03 hoac 04: doc `so_luong` thanh ghi tu `dia_chi`.
    Doc { ma_ham: u8, dia_chi: u16, so_luong: u16 },
    /// Ma ham 06.
    GhiMot { dia_chi: u16, gia_tri: u16 },
    /// Ma ham 16. `du_lieu` dai dung `2 * so_luong` byte, byte cao truoc.
    GhiNhieu { dia_chi: u16, so_luong: u16, du_lieu: &'a [u8] },
}

impl YeuCau<'_> {
    pub fn ma_ham(&self) -> u8 {
        match self {
            YeuCau::Doc { ma_ham, .. } => *ma_ham,
            YeuCau::GhiMot { .. } => GHI_MOT,
            YeuCau::GhiNhieu { .. } => GHI_NHIEU,
        }
    }
}

/// Gia tri thanh ghi thu `i` cua mot yeu cau ghi nhieu.
pub fn thanh_ghi(du_lieu: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([du_lieu[2 * i], du_lieu[2 * i + 1]])
}

/// Vi sao mot khung bi bo, khong tra loi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LyDoBo {
    /// Gui cho slave khac.
    DiaChiKhac,
    SaiCrc,
    /// Dai hon do dai cua ma ham.
    ThuaByte,
    /// Ma ham 16 khai hon 246 byte du lieu.
    QuaDai,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KetQua<'a> {
    /// Khung chua du: cho them byte (slave bo khung neu cho qua lau).
    ThieuByte,
    BoQua(LyDoBo),
    /// Tra loi ma ngoai le `ma` — tru khi la quang ba.
    NgoaiLe { ma_ham: u8, ma: u8, quang_ba: bool },
    /// Yeu cau hop le. Quang ba thi van lam, nhung khong tra loi.
    YeuCau { yc: YeuCau<'a>, quang_ba: bool },
}

fn u16_be(k: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([k[i], k[i + 1]])
}

fn crc_dung(k: &[u8]) -> bool {
    let n = k.len();
    n >= 4 && crc(&k[..n - 2]) == u16::from_le_bytes([k[n - 2], k[n - 1]])
}

/// Tach mot khung yeu cau gui toi slave co dia chi `dia_chi` (1..247).
pub fn tach(khung: &[u8], dia_chi: u8) -> KetQua<'_> {
    let Some(&dich) = khung.first() else {
        return KetQua::ThieuByte;
    };
    if dich != dia_chi && dich != QUANG_BA {
        return KetQua::BoQua(LyDoBo::DiaChiKhac);
    }
    let Some(&ma_ham) = khung.get(1) else {
        return KetQua::ThieuByte;
    };
    let quang_ba = dich == QUANG_BA;

    let dai = match ma_ham {
        DOC_HOLDING | DOC_INPUT | GHI_MOT => 8,
        GHI_NHIEU => {
            let Some(&so_byte) = khung.get(6) else {
                return KetQua::ThieuByte;
            };
            if so_byte > 246 {
                return KetQua::BoQua(LyDoBo::QuaDai);
            }
            9 + so_byte as usize
        }
        _ => {
            // Ma ham khong ho tro: khong biet do dai, ca bo dem la mot khung.
            return if crc_dung(khung) {
                KetQua::NgoaiLe { ma_ham, ma: NL_MA_HAM, quang_ba }
            } else {
                KetQua::BoQua(LyDoBo::SaiCrc)
            };
        }
    };
    if khung.len() < dai {
        return KetQua::ThieuByte;
    }
    if khung.len() > dai {
        return KetQua::BoQua(LyDoBo::ThuaByte);
    }
    if !crc_dung(khung) {
        return KetQua::BoQua(LyDoBo::SaiCrc);
    }

    let dia_chi_tg = u16_be(khung, 2);
    let ngoai_le = |ma| KetQua::NgoaiLe { ma_ham, ma, quang_ba };
    let tran = |so_luong: u16| dia_chi_tg as u32 + so_luong as u32 > 0x1_0000;
    let yc = match ma_ham {
        DOC_HOLDING | DOC_INPUT => {
            let so_luong = u16_be(khung, 4);
            if !(1..=DOC_TOI_DA).contains(&so_luong) {
                return ngoai_le(NL_GIA_TRI);
            }
            if tran(so_luong) {
                return ngoai_le(NL_DIA_CHI);
            }
            YeuCau::Doc { ma_ham, dia_chi: dia_chi_tg, so_luong }
        }
        GHI_MOT => YeuCau::GhiMot { dia_chi: dia_chi_tg, gia_tri: u16_be(khung, 4) },
        _ => {
            let so_luong = u16_be(khung, 4);
            let du_lieu = &khung[7..dai - 2];
            if !(1..=GHI_TOI_DA).contains(&so_luong) {
                return ngoai_le(NL_GIA_TRI);
            }
            if tran(so_luong) {
                return ngoai_le(NL_DIA_CHI);
            }
            if du_lieu.len() != 2 * so_luong as usize {
                return ngoai_le(NL_GIA_TRI);
            }
            YeuCau::GhiNhieu { dia_chi: dia_chi_tg, so_luong, du_lieu }
        }
    };
    KetQua::YeuCau { yc, quang_ba }
}

/// Ghi CRC vao sau `n` byte dau cua `ra`, tra ve do dai ca khung.
fn them_crc(ra: &mut [u8], n: usize) -> usize {
    let c = crc(&ra[..n]).to_le_bytes();
    ra[n] = c[0];
    ra[n + 1] = c[1];
    n + 2
}

/// Tra loi doc (03/04): `dia_chi | ma_ham | 2n | n thanh ghi | crc`.
/// Tra ve 0 (khong dung gi) neu `n` ngoai 1..=125 hoac `ra` khong du `5 + 2n` byte.
pub fn tra_loi_doc(dia_chi: u8, ma_ham: u8, thanh_ghi: &[u16], ra: &mut [u8]) -> usize {
    let n = thanh_ghi.len();
    if !(1..=DOC_TOI_DA as usize).contains(&n) || ra.len() < 5 + 2 * n {
        return 0;
    }
    ra[0] = dia_chi;
    ra[1] = ma_ham;
    ra[2] = (2 * n) as u8;
    for (i, g) in thanh_ghi.iter().enumerate() {
        ra[3 + 2 * i..5 + 2 * i].copy_from_slice(&g.to_be_bytes());
    }
    them_crc(ra, 3 + 2 * n)
}

/// Tra loi ghi (06/16): `dia_chi | ma_ham | a | b | crc` — 06 lap lai dia chi va gia tri,
/// 16 lap lai dia chi va so luong. 8 byte; 0 neu `ra` ngan hon.
pub fn tra_loi_ghi(dia_chi: u8, ma_ham: u8, a: u16, b: u16, ra: &mut [u8]) -> usize {
    if ra.len() < 8 {
        return 0;
    }
    ra[0] = dia_chi;
    ra[1] = ma_ham;
    ra[2..4].copy_from_slice(&a.to_be_bytes());
    ra[4..6].copy_from_slice(&b.to_be_bytes());
    them_crc(ra, 6)
}

/// Tra loi ngoai le: `dia_chi | ma_ham + 0x80 | ma | crc`. 5 byte; 0 neu `ra` ngan hon.
pub fn ngoai_le(dia_chi: u8, ma_ham: u8, ma: u8, ra: &mut [u8]) -> usize {
    if ra.len() < 5 {
        return 0;
    }
    ra[0] = dia_chi;
    ra[1] = ma_ham | 0x80;
    ra[2] = ma;
    them_crc(ra, 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Khung yeu cau: cac byte dau + CRC.
    fn khung(dau: &[u8]) -> Vec<u8> {
        let mut k = dau.to_vec();
        k.extend_from_slice(&crc(dau).to_le_bytes());
        k
    }

    #[test]
    fn crc_chuan() {
        assert_eq!(crc(b"123456789"), 0x4B37);
        // Vi du kinh dien: 01 03 00 00 00 0A -> CRC C5 CD tren day
        assert_eq!(khung(&[1, 3, 0, 0, 0, 0x0A])[6..], [0xC5, 0xCD]);
    }

    #[test]
    fn doc_holding_hop_le() {
        let k = khung(&[1, 3, 0, 100, 0, 2]);
        assert_eq!(
            tach(&k, 1),
            KetQua::YeuCau { yc: YeuCau::Doc { ma_ham: 3, dia_chi: 100, so_luong: 2 }, quang_ba: false }
        );
    }

    #[test]
    fn dia_chi_khac_va_quang_ba() {
        let k = khung(&[2, 3, 0, 0, 0, 1]);
        assert_eq!(tach(&k, 1), KetQua::BoQua(LyDoBo::DiaChiKhac));
        // chua co ma ham cung biet la cua slave khac: khong phai cho
        assert_eq!(tach(&[2], 1), KetQua::BoQua(LyDoBo::DiaChiKhac));
        let k = khung(&[0, 6, 0, 5, 0x12, 0x34]);
        assert_eq!(
            tach(&k, 1),
            KetQua::YeuCau { yc: YeuCau::GhiMot { dia_chi: 5, gia_tri: 0x1234 }, quang_ba: true }
        );
    }

    #[test]
    fn thieu_byte_va_thua_byte() {
        let k = khung(&[1, 3, 0, 0, 0, 1]);
        for n in 0..k.len() {
            assert_eq!(tach(&k[..n], 1), KetQua::ThieuByte, "{} byte", n);
        }
        let mut dai = k.clone();
        dai.push(0);
        assert_eq!(tach(&dai, 1), KetQua::BoQua(LyDoBo::ThuaByte));
    }

    #[test]
    fn sai_crc() {
        let mut k = khung(&[1, 4, 0, 0, 0, 1]);
        k[7] ^= 1;
        assert_eq!(tach(&k, 1), KetQua::BoQua(LyDoBo::SaiCrc));
    }

    #[test]
    fn ngoai_le_giong_nanomodbus() {
        let nl = |ma_ham, ma| KetQua::NgoaiLe { ma_ham, ma, quang_ba: false };
        // so luong 0 hoac > 125: gia tri sai (03)
        assert_eq!(tach(&khung(&[1, 3, 0, 0, 0, 0]), 1), nl(3, NL_GIA_TRI));
        assert_eq!(tach(&khung(&[1, 4, 0, 0, 0, 126]), 1), nl(4, NL_GIA_TRI));
        // vuot 0xFFFF: dia chi sai (02); vua cham 0xFFFF thi duoc
        assert_eq!(tach(&khung(&[1, 3, 0xFF, 0xFF, 0, 2]), 1), nl(3, NL_DIA_CHI));
        assert!(matches!(tach(&khung(&[1, 3, 0xFF, 0xFF, 0, 1]), 1), KetQua::YeuCau { .. }));
        // ghi nhieu: so byte khong khop so luong, so luong > 123, so byte 0
        assert_eq!(tach(&khung(&[1, 16, 0, 0, 0, 2, 2, 0, 1]), 1), nl(16, NL_GIA_TRI));
        let mut dau = vec![1, 16, 0, 0, 0, 124, 248];
        dau.resize(7 + 248, 0);
        assert_eq!(tach(&khung(&dau), 1), KetQua::BoQua(LyDoBo::QuaDai));
        assert_eq!(tach(&khung(&[1, 16, 0, 0, 0, 0, 0]), 1), nl(16, NL_GIA_TRI));
        // ma ham khong ho tro, CRC dung: 01; CRC sai: bo
        assert_eq!(tach(&khung(&[1, 5, 0, 0, 0xFF, 0]), 1), nl(5, NL_MA_HAM));
        let mut k = khung(&[1, 5, 0, 0, 0xFF, 0]);
        k[2] ^= 1;
        assert_eq!(tach(&k, 1), KetQua::BoQua(LyDoBo::SaiCrc));
    }

    #[test]
    fn ghi_nhieu_hop_le() {
        let k = khung(&[1, 16, 0, 100, 0, 2, 4, 0, 111, 0, 222]);
        let KetQua::YeuCau { yc: YeuCau::GhiNhieu { dia_chi, so_luong, du_lieu }, quang_ba } = tach(&k, 1) else {
            panic!("phai la ghi nhieu");
        };
        assert_eq!((dia_chi, so_luong, quang_ba), (100, 2, false));
        assert_eq!((thanh_ghi(du_lieu, 0), thanh_ghi(du_lieu, 1)), (111, 222));
    }

    #[test]
    fn dung_cau_tra_loi() {
        let mut ra = [0u8; 16];
        let n = tra_loi_doc(1, 3, &[0x0102, 3, 777], &mut ra);
        assert_eq!(&ra[..n - 2], &[1, 3, 6, 1, 2, 0, 3, 3, 9]);
        assert_eq!(crc(&ra[..n]), 0); // CRC dung thi CRC ca khung = 0
        let n = tra_loi_ghi(1, 6, 100, 55, &mut ra);
        assert_eq!(&ra[..n], &khung(&[1, 6, 0, 100, 0, 55])[..]);
        let n = ngoai_le(1, 3, NL_DIA_CHI, &mut ra);
        assert_eq!(&ra[..n], &khung(&[1, 0x83, 2])[..]);
        // bo dem thieu, so luong sai: khong dung, khong panic
        assert_eq!(tra_loi_doc(1, 3, &[1, 2, 3], &mut ra[..10]), 0);
        assert_eq!(tra_loi_doc(1, 3, &[], &mut ra), 0);
        assert_eq!(ngoai_le(1, 3, 2, &mut ra[..4]), 0);
    }
}
