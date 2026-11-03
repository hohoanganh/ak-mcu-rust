//! Quay man hinh cua cac bai demo tren may tinh, ghi tung khung ra file `<bai>.frames`
//! (moi khung 1024 byte = bo nho SSD1309, 20 khung/giay), giong cach ak-mcu-base lam voi
//! `test_demo`. Sau do `tools/demo_gif.py` dung thanh GIF.
//!
//!   cargo run --release -p ak-hinh --example quay --target x86_64-pc-windows-gnu -- target/quay
//!   python tools/demo_gif.py target/quay docs/anh
//!
//! Hinh la hinh do chinh cac ham `ak_hinh::ve_*` ve — cung ma voi ma chay tren kit. Chi co
//! "nguoi bam nut" la kich ban viet tay o day.

use ak_hinh::{ve_an_toan, ve_ba_viec, ve_dong_ho, ve_nut_bam, ve_panic, Ferris, KhungHinh};
use std::{env, fs, path::{Path, PathBuf}};

const MS_MOI_KHUNG: u64 = 50; // 20 khung/giay

struct Quay {
    byte: Vec<u8>,
}

impl Quay {
    fn new() -> Self {
        Quay { byte: Vec::new() }
    }

    /// Ve mot khung bang `ve`, giu no `ms` mili giay.
    fn giu(&mut self, ms: u64, ve: impl FnOnce(&mut KhungHinh)) {
        let mut khung = KhungHinh::new();
        ve(&mut khung);
        for _ in 0..(ms / MS_MOI_KHUNG).max(1) {
            self.byte.extend_from_slice(&khung.byte);
        }
    }

    fn luu(&self, thu_muc: &Path, ten: &str) {
        let duong = thu_muc.join(format!("{ten}.frames"));
        fs::write(&duong, &self.byte).unwrap();
        println!("{ten}: {} khung, {:.1} s", self.byte.len() / KhungHinh::new().byte.len(), self.byte.len() as f64 / 1024.0 * MS_MOI_KHUNG as f64 / 1000.0);
    }
}

/// Bai 2: LED 300 ms, coi 1 s, OLED ve lai moi 100 ms — dong ho dem 12 giay.
fn bai2() -> Quay {
    let mut q = Quay::new();
    for ms in (0..12_000).step_by(100) {
        q.giu(100, |k| ve_ba_viec(k, ms));
    }
    q
}

/// Bai 3: bam lan luot cac nut, moi giay mot lan.
fn bai3() -> Quay {
    let mut q = Quay::new();
    q.giu(1500, |k| ve_nut_bam(k, "BAM NUT", 0));
    let nut = ["DO", "MI", "SOL", "SOL", "MI", "DO", "DO", "MI"];
    for (i, ten) in nut.iter().enumerate() {
        q.giu(1000, |k| ve_nut_bam(k, ten, i as u32 + 1));
    }
    q
}

/// Bai 4: dong ho kim chay tu 10:09:50, qua moc 10:10:00.
fn bai4() -> Quay {
    let mut q = Quay::new();
    let (mut gio, mut phut, mut giay) = (10u8, 9u8, 50u8);
    for _ in 0..16 {
        q.giu(1000, |k| ve_dong_ho(k, gio, phut, giay));
        giay += 1;
        if giay == 60 {
            giay = 0;
            phut += 1;
        }
        if phut == 60 {
            phut = 0;
            gio += 1;
        }
    }
    q
}

/// Dong gay panic cua bai 5 (`bai/an-toan/src/main.rs`): `diem[chi_so]`.
const DONG_PANIC_BAI5: u32 = 34;

/// Bai 5: doc tung phan tu, ▲ qua chi so 4, roi doc o chi so 5 -> man hinh PANIC.
fn bai5() -> Quay {
    let mut q = Quay::new();
    let diem = [7, 9, 5, 10, 8];
    let kich_ban: [(usize, Option<i32>); 9] = [
        (0, None),
        (0, Some(diem[0])),
        (1, None),
        (1, Some(diem[1])),
        (2, None),
        (3, None),
        (4, None),
        (4, Some(diem[4])),
        (5, None), // ▲ qua cuoi mang: man hinh doi goi y thanh "NGOAI MANG!"
    ];
    for (chi_so, vua_doc) in kich_ban {
        q.giu(900, |k| ve_an_toan(k, chi_so, diem.len(), vua_doc));
    }
    q.giu(900, |k| ve_an_toan(k, 5, diem.len(), None));
    q.giu(3500, |k| {
        ve_panic(k, "index out of bounds: the len is 5 but the index is 5", "bai/an-toan/src/main.rs", DONG_PANIC_BAI5)
    });
    q
}

/// Bai 6: Ferris tu choi 15 giay. Nguoi choi gia lap: chay ve phia duoi sao, thinh thoang nhay.
fn bai6() -> Quay {
    let mut q = Quay::new();
    let mut tro_choi = Ferris::new(0xC0FFEE);
    let (mut gio_game, mut gio_quay, mut buoc) = (0u64, 0u64, 0u32);
    while gio_quay < 15_000 {
        while gio_game + 33 <= gio_quay {
            let (x, sao_x) = tro_choi.vi_tri();
            let muc_tieu = sao_x - 10;
            let nhay = buoc % 45 == 20;
            tro_choi.buoc(x > muc_tieu + 2, x < muc_tieu - 2, nhay);
            gio_game += 33;
            buoc += 1;
        }
        q.giu(MS_MOI_KHUNG, |k| tro_choi.ve(k));
        gio_quay += MS_MOI_KHUNG;
    }
    q
}

fn main() {
    let thu_muc = PathBuf::from(env::args().nth(1).expect("dung: quay <thu muc ra>"));
    fs::create_dir_all(&thu_muc).unwrap();
    bai2().luu(&thu_muc, "bai2-ba-viec-cung-luc");
    bai3().luu(&thu_muc, "bai3-nut-bam");
    bai4().luu(&thu_muc, "bai4-dong-ho");
    bai5().luu(&thu_muc, "bai5-an-toan");
    bai6().luu(&thu_muc, "bai6-cua-ferris");
}
