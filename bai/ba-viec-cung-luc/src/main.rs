//! Bai 2 — Ba viec cung luc, khong can RTOS.
//!
//! Ba task, ba nhip khac nhau, chay chung mot loi CPU:
//!   - LED doi trang thai moi 300 ms
//!   - coi keu "tich" moi giay
//!   - man hinh OLED dem thoi gian, ve lai moi 100 ms
//!
//! Khong task nao phai doi task nao: moi cho `.await` la mot cho nhuong luot.
//! Hinh ve nam o `ak_hinh::ve_ba_viec` (cung ma do dung ra GIF tren may tinh).

#![no_std]
#![no_main]

use ak_hinh::ve_ba_viec;
use ak_kit::{Coi, Led, ManHinh};
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};

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
    let bat_dau = Instant::now();

    loop {
        man_hinh.xoa();
        ve_ba_viec(&mut man_hinh, bat_dau.elapsed().as_millis());
        man_hinh.cap_nhat();

        Timer::after_millis(100).await;
    }
}
