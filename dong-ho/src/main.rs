//! Bai 4 — Cac task noi chuyen qua channel.
//!
//! Task `doc_dong_ho` doc RTC va gui gio moi vao KENH moi khi so giay doi.
//! Task `hien_thi` nhan tu KENH, ve dong ho kim va gio so len OLED.
//! Hai task khong dung chung bien nao: chi trao doi qua channel —
//! giong message cua kernel AK trong firmware C.

#![no_std]
#![no_main]

use ak_kit::{in_ra, DongHo, Gio, LoiDongHo, ManHinh};
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use micromath::F32Ext;

/// Hang doi giua hai task, chua toi da 4 lan gio.
static KENH: Channel<CriticalSectionRawMutex, Gio, 4> = Channel::new();

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let kit = ak_kit::init();
    spawner.spawn(doc_dong_ho(kit.dong_ho).unwrap());
    spawner.spawn(hien_thi(kit.man_hinh).unwrap());
}

#[embassy_executor::task]
async fn doc_dong_ho(mut dong_ho: DongHo) {
    let mut truoc: Option<Gio> = None;
    loop {
        match dong_ho.doc() {
            Ok(gio) if Some(gio) != truoc => {
                truoc = Some(gio);
                KENH.send(gio).await; // gui sang task hien thi
            }
            Ok(_) => {} // chua sang giay moi
            Err(LoiDongHo::MatGio) => {
                in_ra!("RTC mat gio, dat lai 00:00:00");
                dong_ho.dat(Gio { gio: 0, phut: 0, giay: 0 }).ok();
            }
            Err(loi) => in_ra!("loi doc dong ho: {:?}", loi),
        }
        Timer::after_millis(200).await;
    }
}

#[embassy_executor::task]
async fn hien_thi(mut man_hinh: ManHinh) {
    loop {
        let gio = KENH.receive().await; // cho toi khi co gio moi
        in_ra!("{:02}:{:02}:{:02}", gio.gio, gio.phut, gio.giay);
        ve(&mut man_hinh, gio);
    }
}

fn ve(man_hinh: &mut ManHinh, g: Gio) {
    let net = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let tam = Point::new(31, 32);
    man_hinh.xoa();

    // Mat dong ho va 12 vach
    Circle::with_center(tam, 61).into_styled(net).draw(man_hinh).unwrap();
    for i in 0..12 {
        Line::new(diem(tam, i as f32 / 12.0, 26.0), diem(tam, i as f32 / 12.0, 29.0))
            .into_styled(net)
            .draw(man_hinh)
            .unwrap();
    }
    // Ba kim: gio (ngan, dam), phut, giay (dai, manh)
    let phut = g.phut as f32 + g.giay as f32 / 60.0;
    let gio = (g.gio % 12) as f32 + phut / 60.0;
    let dam = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    let vua = PrimitiveStyle::with_stroke(BinaryColor::On, 2);
    Line::new(tam, diem(tam, gio / 12.0, 15.0)).into_styled(dam).draw(man_hinh).unwrap();
    Line::new(tam, diem(tam, phut / 60.0, 22.0)).into_styled(vua).draw(man_hinh).unwrap();
    Line::new(tam, diem(tam, g.giay as f32 / 60.0, 27.0)).into_styled(net).draw(man_hinh).unwrap();

    // Gio so ben phai
    let mut chu: heapless::String<8> = heapless::String::new();
    write!(chu, "{:02}:{:02}", g.gio, g.phut).unwrap();
    Text::new(&chu, Point::new(68, 30), MonoTextStyle::new(&FONT_10X20, BinaryColor::On)).draw(man_hinh).unwrap();
    chu.clear();
    write!(chu, ":{:02}", g.giay).unwrap();
    Text::new(&chu, Point::new(98, 46), MonoTextStyle::new(&FONT_6X10, BinaryColor::On)).draw(man_hinh).unwrap();

    man_hinh.cap_nhat();
}

/// Diem tren vong tron ban kinh `r`, o vi tri `phan` cua mot vong (0 = 12 gio, quay theo chieu kim).
fn diem(tam: Point, phan: f32, r: f32) -> Point {
    let goc = phan * 2.0 * core::f32::consts::PI;
    Point::new(tam.x + (goc.sin() * r) as i32, tam.y - (goc.cos() * r) as i32)
}
