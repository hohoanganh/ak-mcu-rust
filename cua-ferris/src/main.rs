//! Bai 6 — Ferris hung sao. Rust ve muot tren chip 32 MHz.
//!
//! ▼ di trai, ⏎ di phai, ▲ nhay. Sao roi tu tren xuong: hung duoc +1 diem.
//! Hinh Ferris (linh vat cua Rust) ve bang diem anh ngay trong ma nguon.
//! Am thanh chay o task rieng, nhan lenh qua channel: coi keu ma khung hinh khong khung.

#![no_std]
#![no_main]

use ak_kit::{Coi, ManHinh};
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;

/// Ferris, 22 x 14 diem anh: '#' = sang.
const FERRIS: [&str; 14] = [
    ".##................##.",
    "#..#..............#..#",
    "#.##..............##.#",
    ".##...#........#...##.",
    "..#..###......###..#..",
    "..##..#........#..##..",
    "...################...",
    "..##################..",
    ".####################.",
    ".#####.########.#####.",
    "..####..######..####..",
    "..#..##........##..#..",
    ".#..#............#..#.",
    "#..#..............#..#",
];

const MAT_DAT: i32 = 62;

/// Lenh am thanh: (tan so Hz, thoi gian ms)
static AM_THANH: Channel<CriticalSectionRawMutex, (u32, u64), 4> = Channel::new();

#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut kit = ak_kit::init(spawner);
    spawner.spawn(phat_am_thanh(kit.coi).unwrap());

    let (mut x, mut y, mut van_toc_y) = (53i32, (MAT_DAT - 14) as f32, 0.0f32);
    let mut ngau_nhien = Instant::now().as_ticks() as u32 | 1;
    let (mut sao_x, mut sao_y) = (64i32, 0i32);
    let mut diem: u32 = 0;

    loop {
        // Doc nut moi khung hinh (30 khung/giay)
        if kit.nut_xuong.dang_bam() { x = (x - 2).max(0); }
        if kit.nut_ok.dang_bam() { x = (x + 2).min(128 - 22); }
        let tren_dat = y >= (MAT_DAT - 14) as f32;
        if kit.nut_len.dang_bam() && tren_dat {
            van_toc_y = -4.5;
            AM_THANH.try_send((1200, 40)).ok();
        }
        // Trong luc: keo Ferris ve mat dat
        van_toc_y += 0.35;
        y = (y + van_toc_y).min((MAT_DAT - 14) as f32);

        // Sao roi; cham Ferris thi +1, cham dat thi mat
        sao_y += 2;
        let trung = sao_x + 2 >= x && sao_x <= x + 20 && sao_y + 2 >= y as i32 && sao_y <= y as i32 + 13;
        if trung || sao_y > MAT_DAT {
            if trung {
                diem += 1;
                AM_THANH.try_send((1800, 60)).ok();
            } else {
                AM_THANH.try_send((300, 80)).ok();
            }
            ngau_nhien ^= ngau_nhien << 13;
            ngau_nhien ^= ngau_nhien >> 17;
            ngau_nhien ^= ngau_nhien << 5;
            sao_x = (ngau_nhien % 120) as i32 + 2;
            sao_y = 10;
        }

        ve(&mut kit.man_hinh, x, y as i32, sao_x, sao_y, diem);
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

fn ve(man_hinh: &mut ManHinh, x: i32, y: i32, sao_x: i32, sao_y: i32, diem: u32) {
    man_hinh.xoa();

    // Ferris: moi '#' la mot diem sang
    let diem_anh = FERRIS.iter().enumerate().flat_map(|(hang, dong)| {
        dong.bytes().enumerate().filter(|&(_, c)| c == b'#').map(move |(cot, _)| {
            Pixel(Point::new(x + cot as i32, y + hang as i32), BinaryColor::On)
        })
    });
    man_hinh.draw_iter(diem_anh).unwrap();

    // Ngoi sao hinh dau cong 5x5, mat dat, diem so
    let net = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    Line::new(Point::new(sao_x - 2, sao_y), Point::new(sao_x + 2, sao_y)).into_styled(net).draw(man_hinh).unwrap();
    Line::new(Point::new(sao_x, sao_y - 2), Point::new(sao_x, sao_y + 2)).into_styled(net).draw(man_hinh).unwrap();
    Line::new(Point::new(0, MAT_DAT + 1), Point::new(127, MAT_DAT + 1)).into_styled(net).draw(man_hinh).unwrap();
    let mut chu: heapless::String<12> = heapless::String::new();
    write!(chu, "sao: {}", diem).unwrap();
    Text::new(&chu, Point::new(0, 7), MonoTextStyle::new(&FONT_6X10, BinaryColor::On)).draw(man_hinh).unwrap();

    man_hinh.cap_nhat();
}
