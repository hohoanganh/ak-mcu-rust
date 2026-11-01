//! Test tinh chat cho bo tach khung Modbus RTU: proptest sinh hang nghin khung ngau nhien
//! va kiem nhung dieu phai LUON dung.

use ak_modbus::*;
use proptest::prelude::*;

/// Mot yeu cau hop le bat ky, kem dia chi gui toi (slave 1 hoac quang ba).
#[derive(Debug, Clone)]
struct Mau {
    dich: u8,
    ma_ham: u8,
    dia_chi: u16,
    so_luong: u16,
    gia_tri: Vec<u16>,
}

fn mau() -> impl Strategy<Value = Mau> {
    let dich = prop_oneof![9 => Just(1u8), 1 => Just(QUANG_BA)];
    let doc = (dich.clone(), prop_oneof![Just(DOC_HOLDING), Just(DOC_INPUT)], 1u16..=DOC_TOI_DA)
        .prop_flat_map(|(dich, ma_ham, so_luong)| {
            (0..=(0x1_0000 - so_luong as u32) as u16).prop_map(move |dia_chi| Mau {
                dich,
                ma_ham,
                dia_chi,
                so_luong,
                gia_tri: vec![],
            })
        });
    let ghi_mot = (dich.clone(), any::<u16>(), any::<u16>()).prop_map(|(dich, dia_chi, v)| Mau {
        dich,
        ma_ham: GHI_MOT,
        dia_chi,
        so_luong: 1,
        gia_tri: vec![v],
    });
    let ghi_nhieu = (dich, prop::collection::vec(any::<u16>(), 1..=GHI_TOI_DA as usize)).prop_flat_map(
        |(dich, gia_tri)| {
            let n = gia_tri.len() as u32;
            (0..=(0x1_0000 - n) as u16).prop_map(move |dia_chi| Mau {
                dich,
                ma_ham: GHI_NHIEU,
                dia_chi,
                so_luong: gia_tri.len() as u16,
                gia_tri: gia_tri.clone(),
            })
        },
    );
    prop_oneof![doc, ghi_mot, ghi_nhieu]
}

/// Dong goi yeu cau thanh byte tren day (nhu master).
fn dong_goi(m: &Mau) -> Vec<u8> {
    let mut k = vec![m.dich, m.ma_ham];
    k.extend_from_slice(&m.dia_chi.to_be_bytes());
    match m.ma_ham {
        GHI_MOT => k.extend_from_slice(&m.gia_tri[0].to_be_bytes()),
        GHI_NHIEU => {
            k.extend_from_slice(&m.so_luong.to_be_bytes());
            k.push(2 * m.gia_tri.len() as u8);
            for v in &m.gia_tri {
                k.extend_from_slice(&v.to_be_bytes());
            }
        }
        _ => k.extend_from_slice(&m.so_luong.to_be_bytes()),
    }
    let c = crc(&k).to_le_bytes();
    k.extend_from_slice(&c);
    k
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// Moi yeu cau hop le dong goi ra deu tach lai dung tung truong.
    #[test]
    fn dong_goi_roi_tach_lai_dung(m in mau()) {
        let k = dong_goi(&m);
        let KetQua::YeuCau { yc, quang_ba } = tach(&k, 1) else {
            return Err(TestCaseError::fail(format!("khong ra yeu cau: {:?}", tach(&k, 1))));
        };
        prop_assert_eq!(quang_ba, m.dich == QUANG_BA);
        prop_assert_eq!(yc.ma_ham(), m.ma_ham);
        match yc {
            YeuCau::Doc { dia_chi, so_luong, .. } => prop_assert_eq!((dia_chi, so_luong), (m.dia_chi, m.so_luong)),
            YeuCau::GhiMot { dia_chi, gia_tri } => prop_assert_eq!((dia_chi, gia_tri), (m.dia_chi, m.gia_tri[0])),
            YeuCau::GhiNhieu { dia_chi, so_luong, du_lieu } => {
                prop_assert_eq!((dia_chi, so_luong), (m.dia_chi, m.so_luong));
                let v: Vec<u16> = (0..so_luong as usize).map(|i| thanh_ghi(du_lieu, i)).collect();
                prop_assert_eq!(v, m.gia_tri.clone());
            }
        }
    }

    /// Lat mot bit bat ky: khong bao gio thanh mot yeu cau hay mot cau tra loi. Bit lat
    /// roi vao do dai thi khung thieu/thua byte; con lai CRC bat duoc moi loi mot bit.
    #[test]
    fn lat_mot_bit_thi_khong_lam_gi(m in mau(), chon: prop::sample::Index, bit in 0u8..8) {
        let mut k = dong_goi(&m);
        let i = chon.index(k.len());
        k[i] ^= 1 << bit;
        let kq = tach(&k, 1);
        prop_assert!(matches!(kq, KetQua::ThieuByte | KetQua::BoQua(_)), "byte {}: {:?}", i, kq);
    }

    /// Thieu byte cuoi bat ky: luon la "chua du", khong bao gio doan.
    #[test]
    fn khung_chua_du_thi_cho(m in mau(), cat: prop::sample::Index) {
        let k = dong_goi(&m);
        let n = cat.index(k.len()); // 0..len-1
        prop_assert_eq!(tach(&k[..n], 1), KetQua::ThieuByte);
    }

    /// Byte bat ky, dia chi bat ky: khong panic; yeu cau nao tach ra cung hop le
    /// (CRC dung, so luong trong gioi han, khong vuot 0xFFFF, du lieu khop so luong).
    #[test]
    fn byte_rac_khong_lam_sap(rac in prop::collection::vec(any::<u8>(), 0..300), dia_chi in 1u8..=247) {
        if let KetQua::YeuCau { yc, .. } = tach(&rac, dia_chi) {
            prop_assert_eq!(crc(&rac), 0); // ca khung kem CRC dung thi CRC = 0
            match yc {
                YeuCau::Doc { dia_chi, so_luong, .. } => {
                    prop_assert!((1..=DOC_TOI_DA).contains(&so_luong));
                    prop_assert!(dia_chi as u32 + so_luong as u32 <= 0x1_0000);
                }
                YeuCau::GhiNhieu { dia_chi, so_luong, du_lieu } => {
                    prop_assert!((1..=GHI_TOI_DA).contains(&so_luong));
                    prop_assert!(dia_chi as u32 + so_luong as u32 <= 0x1_0000);
                    prop_assert_eq!(du_lieu.len(), 2 * so_luong as usize);
                }
                YeuCau::GhiMot { .. } => {}
            }
        }
    }

    /// Cau tra loi nao dung ra cung co CRC dung, va tra loi doc mang dung so thanh ghi.
    #[test]
    fn cau_tra_loi_luon_dung_crc(
        dia_chi in 1u8..=247, regs in prop::collection::vec(any::<u16>(), 0..130),
        a: u16, b: u16, ma in 1u8..=4,
    ) {
        let mut ra = [0u8; KHUNG_TOI_DA];
        let n = tra_loi_doc(dia_chi, DOC_HOLDING, &regs, &mut ra);
        if (1..=DOC_TOI_DA as usize).contains(&regs.len()) {
            prop_assert_eq!(n, 5 + 2 * regs.len());
            prop_assert_eq!(crc(&ra[..n]), 0);
            prop_assert_eq!(ra[2] as usize, 2 * regs.len());
        } else {
            prop_assert_eq!(n, 0);
        }
        let n = tra_loi_ghi(dia_chi, GHI_NHIEU, a, b, &mut ra);
        prop_assert_eq!((n, crc(&ra[..n])), (8, 0));
        let n = ngoai_le(dia_chi, DOC_INPUT, ma, &mut ra);
        prop_assert_eq!((n, crc(&ra[..n]), ra[1]), (5, 0, DOC_INPUT | 0x80));
    }
}
