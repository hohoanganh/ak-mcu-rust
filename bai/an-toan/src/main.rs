//! Bai 5 — Loi C am tham, Rust dung lai va chi dung dong.
//!
//! Mang `diem` co 5 phan tu (chi so 0..4). ▲ tang chi so — co tinh khong chan o 4 —,
//! ▼ giam, ⏎ doc `diem[chi_so]`.
//! O chi so 5, C se doc o nho ben canh va chay tiep voi gia tri rac.
//! Rust kiem gioi han mang moi lan doc: dung lai, man hinh hien PANIC kem ten tep, so dong.
//! Hinh ve nam o `ak_hinh::ve_an_toan` va `ak_hinh::ve_panic`: cung ma do dung ra GIF
//! tren may tinh. Ben duoi chi con luat choi va dong gay panic.
//! Dong gay panic la dong 34 cua file nay: README, tai lieu va clip deu tro toi so do —
//! them hay bot dong o phia tren thi phai sua cac cho do.

#![no_std]
#![no_main]

use ak_hinh::ve_an_toan;
use ak_kit::{in_ra, ManHinh};
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut kit = ak_kit::init(spawner);
    let diem = [7, 9, 5, 10, 8]; // 5 phan tu: chi so hop le 0..4
    let mut chi_so: usize = 0;
    let mut vua_doc: Option<i32> = None;

    loop {
        ve(&mut kit.man_hinh, chi_so, diem.len(), vua_doc);
        match select3(kit.nut_xuong.cho_bam(), kit.nut_len.cho_bam(), kit.nut_ok.cho_bam()).await {
            Either3::First(_) => chi_so = chi_so.saturating_sub(1),
            Either3::Second(_) => chi_so += 1, // co tinh khong chan tren
            Either3::Third(_) => {
                in_ra!("doc diem[{}]", chi_so);
                let gia_tri = diem[chi_so]; // chi_so = 5: Rust dung lai o dong nay
                vua_doc = Some(gia_tri);
                kit.coi.keu(880, 80).await;
                continue;
            }
        }
        vua_doc = None;
    }
}

fn ve(man_hinh: &mut ManHinh, chi_so: usize, do_dai: usize, vua_doc: Option<i32>) {
    man_hinh.xoa();
    ve_an_toan(man_hinh, chi_so, do_dai, vua_doc);
    man_hinh.cap_nhat();
}
