//! Bai 6: tro choi Ferris hung sao. Phan luat choi va phan ve khong biet gi ve chip, nut hay
//! coi: ben goi truyen trang thai ba nut moi khung hinh, nhan lai cac tieng can phat.

use core::fmt::Write;
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
const CAO_FERRIS: f32 = 14.0;

/// Mot tieng cho coi: (tan so Hz, thoi gian ms).
pub type AmThanh = (u32, u64);

pub struct Ferris {
    x: i32,
    y: f32,
    van_toc_y: f32,
    ngau_nhien: u32,
    sao_x: i32,
    sao_y: i32,
    diem: u32,
}

impl Ferris {
    /// `hat`: so bat ky de rai sao (tren kit: dong ho luc khoi dong).
    pub fn new(hat: u32) -> Self {
        Ferris {
            x: 53,
            y: MAT_DAT as f32 - CAO_FERRIS,
            van_toc_y: 0.0,
            ngau_nhien: hat | 1,
            sao_x: 64,
            sao_y: 0,
            diem: 0,
        }
    }

    pub fn diem(&self) -> u32 {
        self.diem
    }

    /// Vi tri ngang cua Ferris va cua sao dang roi (kich ban GIF dung de dua Ferris toi duoi sao).
    pub fn vi_tri(&self) -> (i32, i32) {
        (self.x, self.sao_x)
    }

    /// Mot khung hinh (33 ms): `trai`, `phai`, `nhay` la ba nut co dang giu khong.
    /// Tra ve toi da hai tieng: tieng nhay, roi tieng an sao / roi trung.
    pub fn buoc(&mut self, trai: bool, phai: bool, nhay: bool) -> [Option<AmThanh>; 2] {
        let mut am = [None, None];
        if trai {
            self.x = (self.x - 2).max(0);
        }
        if phai {
            self.x = (self.x + 2).min(128 - 22);
        }
        let tren_dat = self.y >= MAT_DAT as f32 - CAO_FERRIS;
        if nhay && tren_dat {
            self.van_toc_y = -4.5;
            am[0] = Some((1200, 40));
        }
        // Trong luc: keo Ferris ve mat dat
        self.van_toc_y += 0.35;
        self.y = (self.y + self.van_toc_y).min(MAT_DAT as f32 - CAO_FERRIS);

        // Sao roi; cham Ferris thi +1, cham dat thi mat
        self.sao_y += 2;
        let y = self.y as i32;
        let trung = self.sao_x + 2 >= self.x && self.sao_x <= self.x + 20 && self.sao_y + 2 >= y && self.sao_y <= y + 13;
        if trung || self.sao_y > MAT_DAT {
            am[1] = Some(if trung { (1800, 60) } else { (300, 80) });
            if trung {
                self.diem += 1;
            }
            self.ngau_nhien ^= self.ngau_nhien << 13;
            self.ngau_nhien ^= self.ngau_nhien >> 17;
            self.ngau_nhien ^= self.ngau_nhien << 5;
            self.sao_x = (self.ngau_nhien % 120) as i32 + 2;
            self.sao_y = 10;
        }
        am
    }

    pub fn ve<D: DrawTarget<Color = BinaryColor>>(&self, d: &mut D) {
        // Ferris: moi '#' la mot diem sang
        let (x, y) = (self.x, self.y as i32);
        let diem_anh = FERRIS.iter().enumerate().flat_map(|(hang, dong)| {
            dong.bytes()
                .enumerate()
                .filter(|&(_, c)| c == b'#')
                .map(move |(cot, _)| Pixel(Point::new(x + cot as i32, y + hang as i32), BinaryColor::On))
        });
        d.draw_iter(diem_anh).ok();

        // Ngoi sao hinh dau cong 5x5, mat dat, diem so
        let net = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let (sx, sy) = (self.sao_x, self.sao_y);
        Line::new(Point::new(sx - 2, sy), Point::new(sx + 2, sy)).into_styled(net).draw(d).ok();
        Line::new(Point::new(sx, sy - 2), Point::new(sx, sy + 2)).into_styled(net).draw(d).ok();
        Line::new(Point::new(0, MAT_DAT + 1), Point::new(127, MAT_DAT + 1)).into_styled(net).draw(d).ok();
        let mut chu: heapless::String<12> = heapless::String::new();
        write!(chu, "sao: {}", self.diem).unwrap();
        Text::new(&chu, Point::new(0, 7), MonoTextStyle::new(&FONT_6X10, BinaryColor::On)).draw(d).ok();
    }
}
