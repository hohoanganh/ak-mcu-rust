//! Bai 2 — Ba viec cung luc, khong can RTOS.
//!
//! Ba task, ba nhip khac nhau, chay chung mot loi CPU:
//!   - LED doi trang thai moi 300 ms
//!   - coi keu "tich" moi giay
//!   - man hinh OLED dem thoi gian, ve lai moi 100 ms
//!
//! Khong task nao phai doi task nao: moi cho `.await` la mot cho nhuong luot.

#![no_std]
#![no_main]

use ak_kit::{Coi, Led, ManHinh};
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let kit = ak_kit::init(spawner);

    spawner.spawn(nhay_led(kit.led).unwrap());
    spawner.spawn(tich_tac(kit.coi).unwrap());
    spawner.spawn(dem_gio(kit.man_hinh).unwrap());
}

#[embassy_executor::task]
async fn nhay_led(mut led: Led) {
    loop {
        led.dao();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task]
async fn tich_tac(mut coi: Coi) {
    loop {
        coi.keu(2000, 15).await; // "tich" 15 ms
        Timer::after_millis(985).await;
    }
}

#[embassy_executor::task]
async fn dem_gio(mut man_hinh: ManHinh) {
    let chu_nho = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let chu_to = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let bat_dau = Instant::now();

    loop {
        let ms = bat_dau.elapsed().as_millis();
        let mut so: heapless::String<16> = heapless::String::new();
        write!(so, "{}.{} s", ms / 1000, (ms / 100) % 10).unwrap();

        man_hinh.xoa();
        Text::new("3 viec cung luc", Point::new(19, 10), chu_nho).draw(&mut man_hinh).unwrap();
        Text::new(&so, Point::new(24, 38), chu_to).draw(&mut man_hinh).unwrap();
        Text::new("LED 300ms  COI 1s", Point::new(13, 52), chu_nho).draw(&mut man_hinh).unwrap();
        Text::new("OLED 100ms", Point::new(34, 62), chu_nho).draw(&mut man_hinh).unwrap();
        man_hinh.cap_nhat();

        Timer::after_millis(100).await;
    }
}
