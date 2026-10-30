//! Bai 5 — Loi C am tham, Rust dung lai va chi dung dong.
//!
//! Mang `diem` co 5 phan tu (chi so 0..4). ▲ tang chi so — co tinh khong chan o 4 —,
//! ▼ giam, ⏎ doc `diem[chi_so]`.
//! O chi so 5, C se doc o nho ben canh va chay tiep voi gia tri rac.
//! Rust kiem gioi han mang moi lan doc: dung lai, man hinh hien PANIC kem ten tep, so dong.

#![no_std]
#![no_main]

use ak_kit::{in_ra, ManHinh};
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};
use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut kit = ak_kit::init();
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
    let chu_nho = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let chu_to = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let mut dong: heapless::String<24> = heapless::String::new();

    man_hinh.xoa();
    Text::new("mang co 5 phan tu", Point::new(13, 9), chu_nho).draw(man_hinh).unwrap();
    match vua_doc {
        Some(v) => write!(dong, "diem[{}]={}", chi_so, v),
        None => write!(dong, "diem[{}]", chi_so),
    }
    .unwrap();
    Text::new(&dong, Point::new(64 - dong.len() as i32 * 5, 34), chu_to).draw(man_hinh).unwrap();
    let goi_y = if chi_so < do_dai { "v ^ chon   OK doc" } else { "NGOAI MANG! bam OK?" };
    Text::new(goi_y, Point::new(64 - goi_y.len() as i32 * 3, 56), chu_nho).draw(man_hinh).unwrap();
    man_hinh.cap_nhat();
}
