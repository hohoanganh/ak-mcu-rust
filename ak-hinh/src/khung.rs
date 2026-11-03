//! Bo dem khung 128x64, 1 bit/diem, xep theo trang giong bo nho cua SSD1309:
//! byte `trang * 128 + cot`, bit 0 la diem tren cung cua trang 8 diem.

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

pub const RONG: usize = 128;
pub const CAO: usize = 64;

pub struct KhungHinh {
    pub byte: [u8; RONG * CAO / 8],
}

impl KhungHinh {
    pub const fn new() -> Self {
        KhungHinh { byte: [0; RONG * CAO / 8] }
    }

    pub fn xoa(&mut self) {
        self.byte.fill(0);
    }
}

impl Default for KhungHinh {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginDimensions for KhungHinh {
    fn size(&self) -> Size {
        Size::new(RONG as u32, CAO as u32)
    }
}

impl DrawTarget for KhungHinh {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, diem: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, mau) in diem {
            if p.x < 0 || p.y < 0 || p.x >= RONG as i32 || p.y >= CAO as i32 {
                continue;
            }
            let (x, y) = (p.x as usize, p.y as usize);
            let i = (y / 8) * RONG + x;
            let bit = 1 << (y % 8);
            if mau.is_on() {
                self.byte[i] |= bit;
            } else {
                self.byte[i] &= !bit;
            }
        }
        Ok(())
    }
}
