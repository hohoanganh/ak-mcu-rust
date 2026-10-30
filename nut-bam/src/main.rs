//! Bai 3 — Cho su kien, khong hoi lien tuc.
//!
//! Dan ba phim: moi nut mot not nhac (Do, Mi, Sol), LED chop moi lan bam,
//! man hinh hien nut vua bam va so lan bam.
//! `select3` cho ca ba nut trong mot dong; nut nao bam truoc thi nhanh do chay.
//! Khong co vong lap nao hoi trang thai chan: ngat EXTI danh thuc dung cho dang cho.

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
    let mut so_lan: u32 = 0;
    ve(&mut kit.man_hinh, "BAM NUT", so_lan);

    loop {
        // Cho ca ba nut cung luc
        let (ten, tan_so) = match select3(
            kit.nut_xuong.cho_bam(),
            kit.nut_len.cho_bam(),
            kit.nut_ok.cho_bam(),
        )
        .await
        {
            Either3::First(_) => ("DO", 523),  // nut ▼
            Either3::Second(_) => ("MI", 659), // nut ▲
            Either3::Third(_) => ("SOL", 784), // nut ⏎
        };

        so_lan += 1;
        in_ra!("bam lan {}: {}", so_lan, ten);
        kit.led.bat();
        ve(&mut kit.man_hinh, ten, so_lan);
        kit.coi.keu(tan_so, 150).await;
        kit.led.tat();
    }
}

fn ve(man_hinh: &mut ManHinh, ten: &str, so_lan: u32) {
    let chu_nho = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let chu_to = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let mut dem: heapless::String<20> = heapless::String::new();
    write!(dem, "da bam {} lan", so_lan).unwrap();

    man_hinh.xoa();
    Text::new("Dan 3 phim", Point::new(34, 10), chu_nho).draw(man_hinh).unwrap();
    let x = 64 - (ten.len() as i32 * 10) / 2; // canh giua
    Text::new(ten, Point::new(x, 38), chu_to).draw(man_hinh).unwrap();
    Text::new(&dem, Point::new(64 - dem.len() as i32 * 3, 58), chu_nho).draw(man_hinh).unwrap();
    man_hinh.cap_nhat();
}
