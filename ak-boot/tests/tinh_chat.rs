//! Test tinh chat (property-based) cho ak-boot: thay vi vai vi du viet tay, proptest sinh
//! hang nghin dau vao ngau nhien va kiem mot dieu phai LUON dung. Khi mot dieu bi sai,
//! proptest tu thu nho dau vao xuong vi du nho nhat con lam sai, roi in ra.
//!
//! Chay: `cargo test -p ak-boot --target <target cua may>` (vi du x86_64-unknown-linux-gnu).

use ak_boot::boot_ctrl::{self, BootCtrl, LoiNvm, Nvm};
use ak_boot::khung::{dong_goi, BoTach, Khung, DU_LIEU_TOI_DA, HET_GIO_MS, SOF};
use ak_boot::crc16;
use proptest::prelude::*;

// ---------------------------------------------------------------- khung fw_proto

/// Dong goi mot khung thanh byte.
fn khung_byte(cmd: u8, seq: u8, du_lieu: &[u8]) -> Vec<u8> {
    let mut ra = vec![0u8; 7 + du_lieu.len()];
    let n = dong_goi(cmd, seq, du_lieu, &mut ra);
    ra.truncate(n);
    ra
}

/// Nap lan luot tung byte, byte sau cach byte truoc 1 ms; tra ve moi khung nhan duoc.
fn nap_het(bo: &mut BoTach, byte: &[u8], t0: u32) -> Vec<Khung> {
    byte.iter()
        .enumerate()
        .filter_map(|(i, &b)| bo.nap(b, t0.wrapping_add(i as u32)))
        .collect()
}

fn du_lieu() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..=DU_LIEU_TOI_DA)
}

/// Chu cua shell: moi byte tru A5 (A5 la byte mo dau khung).
fn chu_shell() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>().prop_filter("khong phai A5", |b| *b != SOF), 0..64)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Moi khung dong goi ra deu tach lai dung nguyen ven — ke ca khi co chu cua shell
    /// dung truoc, va dong ho ms dang sap tran qua 0xFFFFFFFF.
    #[test]
    fn dong_goi_roi_tach_lai_nguyen_ven(
        cmd: u8, seq: u8, dl in du_lieu(), truoc in chu_shell(), t0: u32,
    ) {
        let mut byte = truoc;
        byte.extend(khung_byte(cmd, seq, &dl));
        let k = nap_het(&mut BoTach::new(), &byte, t0);
        prop_assert_eq!(k.len(), 1);
        prop_assert_eq!((k[0].cmd, k[0].seq, &k[0].du_lieu[..]), (cmd, seq, &dl[..]));
    }

    /// Byte bat ky, den luc bat ky: bo tach khong bao gio panic, va khung nao no nhan ra
    /// cung hop le (dai <= 132 B, CRC dung). Day la cho C hay tran bo dem nhat.
    #[test]
    fn byte_rac_khong_lam_sap(rac in prop::collection::vec((any::<u8>(), 0u32..1000), 0..2000)) {
        let mut bo = BoTach::new();
        let mut t = 0u32;
        for (b, cach) in rac {
            t = t.wrapping_add(cach);
            let _ = bo.byte_thuoc_khung(b, t);
            if let Some(k) = bo.nap(b, t) {
                prop_assert!(k.du_lieu.len() <= DU_LIEU_TOI_DA);
                let ma = khung_byte(k.cmd, k.seq, &k.du_lieu);
                let n = k.du_lieu.len();
                let mut dau = vec![k.cmd, k.seq, n as u8, (n >> 8) as u8];
                dau.extend_from_slice(&k.du_lieu);
                prop_assert_eq!(u16::from_le_bytes([ma[5 + n], ma[6 + n]]), crc16(&dau));
            }
        }
    }

    /// Sau rac bat ky, chi can im lang qua 500 ms la khung ke tiep chac chan tach duoc.
    #[test]
    fn im_lang_500_ms_thi_bat_lai_duoc(
        rac in prop::collection::vec(any::<u8>(), 0..300), cmd: u8, seq: u8, dl in du_lieu(),
    ) {
        let mut bo = BoTach::new();
        nap_het(&mut bo, &rac, 0);
        let t = rac.len() as u32 + HET_GIO_MS + 1;
        let k = nap_het(&mut bo, &khung_byte(cmd, seq, &dl), t);
        prop_assert_eq!(k.len(), 1);
        prop_assert_eq!(&k[0].du_lieu[..], &dl[..]);
    }

    /// Lat mot bit bat ky trong cmd, seq, du lieu hay CRC: khung bi bo (CRC-16 bat duoc
    /// moi loi mot bit). Khong lat SOF va do dai: hai cho do doi cau truc khung.
    #[test]
    fn lat_mot_bit_thi_khong_nhan(
        cmd: u8, seq: u8, dl in du_lieu(), chon: prop::sample::Index, bit in 0u8..8,
    ) {
        let mut byte = khung_byte(cmd, seq, &dl);
        let cho: Vec<usize> = (1..byte.len()).filter(|&i| i != 3 && i != 4).collect();
        byte[cho[chon.index(cho.len())]] ^= 1 << bit;
        prop_assert!(nap_het(&mut BoTach::new(), &byte, 0).is_empty());
    }
}

// ---------------------------------------------------------------- boot_ctrl

/// EEPROM gia 64 B (hai o 32 B). `cat_sau`: mat dien sau khi ghi duoc ngan ay byte —
/// phan dau o la byte moi, phan sau giu byte cu, roi chip tat.
struct NvmGia {
    o: [u8; 64],
    cat_sau: Option<usize>,
}

impl Nvm for NvmGia {
    fn doc(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), LoiNvm> {
        let o = offset as usize;
        buf.copy_from_slice(&self.o[o..o + buf.len()]);
        Ok(())
    }
    fn ghi(&mut self, offset: u32, du_lieu: &[u8]) -> Result<(), LoiNvm> {
        let o = offset as usize;
        let n = self.cat_sau.map_or(du_lieu.len(), |k| k.min(du_lieu.len()));
        self.o[o..o + n].copy_from_slice(&du_lieu[..n]);
        if self.cat_sau.is_some() {
            return Err(LoiNvm::Ghi); // mat dien: khong con gi chay sau do
        }
        Ok(())
    }
}

fn trang_thai() -> impl Strategy<Value = BootCtrl> {
    (any::<[u8; 8]>(), any::<u32>()).prop_map(|(b, install_count)| BootCtrl {
        cmd: b[0],
        install_attempts: b[1],
        last_result: b[2],
        seq: b[3],
        install_count,
        trial: b[4],
        trial_boots: b[5],
        rolled_back: b[6],
        reserved: b[7],
    })
}

/// Mot o 32 B: hoac rac bat ky, hoac mot trang thai hop le bat ky (seq bat ky).
fn mot_o() -> impl Strategy<Value = [u8; 32]> {
    prop_oneof![
        any::<[u8; 32]>(),
        trang_thai().prop_map(|c| {
            let mut o = [0xFFu8; 32];
            o[..20].copy_from_slice(&c.sang_byte());
            o
        }),
    ]
}

fn eeprom() -> impl Strategy<Value = [u8; 64]> {
    (mot_o(), mot_o()).prop_map(|(a, b)| {
        let mut e = [0u8; 64];
        e[..32].copy_from_slice(&a);
        e[32..].copy_from_slice(&b);
        e
    })
}

/// Bo seq ra khi so sanh: `luu()` tu dat seq.
fn bo_seq(mut c: BootCtrl) -> BootCtrl {
    c.seq = 0;
    c
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Ma hoa roi giai ma ra dung trang thai ban dau.
    #[test]
    fn ma_hoa_giai_ma_khu_nhau(c in trang_thai()) {
        prop_assert_eq!(BootCtrl::tu_byte(&c.sang_byte()), Some(c));
    }

    /// EEPROM chua gi cung vay (rac, mot o hong, hai o seq lech xa): doc khong panic, va sau
    /// mot lan `luu()` thi `doc()` tra dung cai vua luu.
    #[test]
    fn luu_xong_doc_lai_dung(e in eeprom(), c in trang_thai()) {
        let mut nvm = NvmGia { o: e, cat_sau: None };
        let _ = boot_ctrl::doc(&mut nvm);
        let mut moi = c;
        boot_ctrl::luu(&mut nvm, &mut moi).unwrap();
        prop_assert_eq!(bo_seq(boot_ctrl::doc(&mut nvm)), bo_seq(c));
    }

    /// Mat dien giua luc ghi, o BAT KY byte nao: lan khoi dong sau doc ra hoac trang thai
    /// cu, hoac trang thai moi — khong bao gio la mot thu lai giua hai cai.
    #[test]
    fn mat_dien_o_byte_bat_ky_van_con_trang_thai_cu(
        e in eeprom(), cu in trang_thai(), moi in trang_thai(), cat in 0usize..=20,
    ) {
        let mut nvm = NvmGia { o: e, cat_sau: None };
        let mut cu_luu = cu;
        boot_ctrl::luu(&mut nvm, &mut cu_luu).unwrap();

        nvm.cat_sau = Some(cat);
        let mut moi_luu = moi;
        let _ = boot_ctrl::luu(&mut nvm, &mut moi_luu); // mat dien o day
        nvm.cat_sau = None;

        let sau = bo_seq(boot_ctrl::doc(&mut nvm));
        // Ban ghi moi co nam nguyen ven trong EEPROM khong? Ngoai cat == 20, con mot truong
        // hop: byte cu chua kip ghi de tinh co trung byte moi (proptest tim ra: cat = 19, byte
        // CRC cuoi cu trung byte CRC cuoi moi, xac suat 1/256).
        let ban_ghi_moi = moi_luu.sang_byte();
        let moi_nguyen_ven = nvm.o[..20] == ban_ghi_moi || nvm.o[32..52] == ban_ghi_moi;
        if cat == 20 {
            prop_assert!(moi_nguyen_ven);
        }
        if moi_nguyen_ven {
            prop_assert_eq!(sau, bo_seq(moi));
        } else {
            prop_assert_eq!(sau, bo_seq(cu)); // khong bao gio la thu lai giua hai cai
        }
    }
}
