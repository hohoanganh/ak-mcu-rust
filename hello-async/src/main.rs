//! Bai 1 — Cho ma khong chan.
//!
//! Hai viec chay cung luc: LED nhay 4 lan moi giay, va dem giay in ra cong COM.
//! `Timer::after(...).await` nhuong CPU cho viec khac trong luc cho,
//! khong ai phai dung `while` ngoi doi.

#![no_std]
#![no_main]

use ak_kit::{in_ra, Led};
use embassy_executor::Spawner;
use embassy_time::Timer;

// Bo chay task "spin": hoi viec lien tuc, khong cho chip ngu, nen ST-Link
// luon nap lai duoc (dau do cua kit khong co chan NRST).
#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let kit = ak_kit::init();
    in_ra!("Xin chao tu Embassy! (Rust async tren AK MCU KIT)");

    // Viec 1: giao LED cho mot task rieng
    spawner.spawn(nhay_led(kit.led).unwrap());

    // Viec 2: dem giay ngay trong main
    let mut giay: u32 = 0;
    loop {
        Timer::after_secs(1).await;
        giay += 1;
        in_ra!("da chay {} giay", giay);
    }
}

#[embassy_executor::task]
async fn nhay_led(mut led: Led) {
    loop {
        led.dao();
        Timer::after_millis(125).await;
    }
}
