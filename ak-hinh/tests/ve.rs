//! Test phan ve va luat choi tren may tinh: khong can kit.

use ak_hinh::{ve_an_toan, ve_ba_viec, ve_dong_ho, ve_nut_bam, ve_panic, Ferris, KhungHinh, RONG};

fn sang(k: &KhungHinh) -> usize {
    k.byte.iter().map(|b| b.count_ones() as usize).sum()
}

fn ve(f: impl FnOnce(&mut KhungHinh)) -> KhungHinh {
    let mut k = KhungHinh::new();
    f(&mut k);
    k
}

#[test]
fn moi_man_hinh_ve_ra_diem_sang_va_khong_tran_khung() {
    let man = [
        ve(|k| ve_ba_viec(k, 12_345)),
        ve(|k| ve_nut_bam(k, "SOL", 7)),
        ve(|k| ve_dong_ho(k, 10, 9, 50)),
        ve(|k| ve_an_toan(k, 3, 5, Some(10))),
        ve(|k| ve_an_toan(k, 5, 5, None)),
        ve(|k| ve_panic(k, "index out of bounds", "bai/an-toan/src/main.rs", 34)),
    ];
    for (i, k) in man.iter().enumerate() {
        let n = sang(k);
        assert!(n > 100 && n < 128 * 64 / 2 + 1000, "man {i}: {n} diem sang");
    }
}

#[test]
fn dong_ho_kim_doi_theo_gio() {
    let a = ve(|k| ve_dong_ho(k, 12, 0, 0));
    let b = ve(|k| ve_dong_ho(k, 12, 0, 30));
    let c = ve(|k| ve_dong_ho(k, 3, 0, 0));
    assert_ne!(a.byte, b.byte);
    assert_ne!(a.byte, c.byte);
}

#[test]
fn man_panic_co_dai_trang_o_tren_cung() {
    let k = ve(|k| ve_panic(k, "x", "a/b/main.rs", 34));
    // 20 dong dau = trang 0 va trang 1 va 4 dong dau cua trang 2; cot 0 khong co chu
    assert_eq!(k.byte[0], 0xFF);
    assert_eq!(k.byte[RONG], 0xFF);
    assert_eq!(k.byte[2 * RONG] & 0x0F, 0x0F);
}

/// README, tai lieu va clip deu ghi "main.rs dong 34": neu ai them/bot dong o phia tren cua
/// bai 5 thi test nay do de nho sua cac cho do (va hang `DONG_PANIC_BAI5` trong examples/quay.rs).
#[test]
fn bai5_gay_panic_dung_o_dong_34() {
    let duong = concat!(env!("CARGO_MANIFEST_DIR"), "/../bai/an-toan/src/main.rs");
    let ma = std::fs::read_to_string(duong).unwrap();
    let dong_34 = ma.lines().nth(33).unwrap();
    assert!(dong_34.contains("diem[chi_so]"), "dong 34 la: {dong_34}");
}

#[test]
fn ferris_nhay_co_tieng_va_chi_nhay_khi_tren_dat() {
    let mut f = Ferris::new(1);
    assert_eq!(f.buoc(false, false, true)[0], Some((1200, 40)));
    assert_eq!(f.buoc(false, false, true)[0], None, "dang tren khong: khong nhay duoc lan nua");
}

#[test]
fn ferris_roi_sao_thi_luon_co_tieng_va_diem_khong_giam() {
    let mut f = Ferris::new(0xC0FFEE);
    let mut diem_truoc = 0;
    let mut tieng_sao = 0;
    for b in 0..2000u32 {
        let (x, sao_x) = f.vi_tri();
        let am = f.buoc(x > sao_x - 12, x < sao_x - 8, b % 45 == 20);
        if am[1].is_some() {
            tieng_sao += 1;
        }
        assert!(f.diem() >= diem_truoc);
        diem_truoc = f.diem();
        let k = ve(|k| f.ve(k));
        assert!(sang(&k) > 10);
    }
    assert!(tieng_sao > 20, "sao phai roi nhieu lan");
    assert!(f.diem() > 5, "nguoi choi gia lap phai an duoc sao");
}
