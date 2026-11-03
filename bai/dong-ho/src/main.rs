//! Bai 4 — Cac task noi chuyen qua channel.
//!
//! Task `doc_dong_ho` doc RTC va gui gio moi vao KENH moi khi so giay doi.
//! Task `hien_thi` nhan tu KENH, ve dong ho kim va gio so len OLED (`ak_hinh::ve_dong_ho`).
//! Hai task khong dung chung bien nao: chi trao doi qua channel —
//! giong message cua kernel AK trong firmware C.

#![no_std]
#![no_main]

use ak_hinh::ve_dong_ho;
use ak_kit::{in_ra, DongHo, Gio, LoiDongHo, ManHinh};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;

/// Hang doi giua hai task, chua toi da 4 lan gio.
static KENH: Channel<CriticalSectionRawMutex, Gio, 4> = Channel::new();

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let kit = ak_kit::init(spawner);
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
        man_hinh.xoa();
        ve_dong_ho(&mut man_hinh, gio.gio, gio.phut, gio.giay);
        man_hinh.cap_nhat();
    }
}
