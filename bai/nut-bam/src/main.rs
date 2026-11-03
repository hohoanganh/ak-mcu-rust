//! Bai 3 — Cho su kien, khong hoi lien tuc.
//!
//! Dan ba phim: moi nut mot not nhac (Do, Mi, Sol), LED chop moi lan bam,
//! man hinh hien nut vua bam va so lan bam.
//! `select3` cho ca ba nut trong mot dong; nut nao bam truoc thi nhanh do chay.
//! Khong co vong lap nao hoi trang thai chan: ngat EXTI danh thuc dung cho dang cho.

#![no_std]
#![no_main]

use ak_hinh::ve_nut_bam;
use ak_kit::{in_ra, ManHinh};
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut kit = ak_kit::init(spawner);
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
    man_hinh.xoa();
    ve_nut_bam(man_hinh, ten, so_lan);
    man_hinh.cap_nhat();
}
