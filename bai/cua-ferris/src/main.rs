//! Bai 6 — Ferris hung sao. Rust ve muot tren chip 32 MHz.
//!
//! ▼ di trai, ⏎ di phai, ▲ nhay. Sao roi tu tren xuong: hung duoc +1 diem.
//! Luat choi va hinh ve (ke ca hinh Ferris, ve bang diem anh ngay trong ma nguon) nam o
//! `ak_hinh::Ferris`, khong biet gi ve chip: file nay chi doc nut, goi `buoc()` moi khung
//! hinh (30 khung/giay) va chuyen tieng sang coi.
//! Am thanh chay o task rieng, nhan lenh qua channel: coi keu ma khung hinh khong khung.

#![no_std]
#![no_main]

use ak_hinh::Ferris;
use ak_kit::Coi;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

/// Lenh am thanh: (tan so Hz, thoi gian ms)
static AM_THANH: Channel<CriticalSectionRawMutex, (u32, u64), 4> = Channel::new();

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut kit = ak_kit::init(spawner);
    spawner.spawn(phat_am_thanh(kit.coi).unwrap());

    let mut tro_choi = Ferris::new(Instant::now().as_ticks() as u32);
    loop {
        // Doc nut moi khung hinh (30 khung/giay)
        let am = tro_choi.buoc(kit.nut_xuong.dang_bam(), kit.nut_ok.dang_bam(), kit.nut_len.dang_bam());
        for tieng in am.into_iter().flatten() {
            AM_THANH.try_send(tieng).ok();
        }

        kit.man_hinh.xoa();
        tro_choi.ve(&mut kit.man_hinh);
        kit.man_hinh.cap_nhat();
        Timer::after_millis(33).await;
    }
}

#[embassy_executor::task]
async fn phat_am_thanh(mut coi: Coi) {
    loop {
        let (tan_so, ms) = AM_THANH.receive().await;
        coi.keu(tan_so, ms).await;
    }
}
