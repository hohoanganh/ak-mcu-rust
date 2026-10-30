//! Dong ho thoi gian thuc PCF85063 tren I2C1 (PB6 SCL, PB7 SDA), dia chi 0x51.
//!
//! Giay, phut, gio dang BCD tu thanh ghi 0x04 — giong `kit_rtc_get()` cua firmware C.
//! Bit 7 cua thanh ghi giay (OS) = dao dong da tung dung, gio khong con dung.

use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Blocking;

const DIA_CHI: u8 = 0x51;

/// Mot thoi diem trong ngay.
#[derive(Clone, Copy, PartialEq)]
pub struct Gio {
    pub gio: u8,
    pub phut: u8,
    pub giay: u8,
}

#[derive(Debug)]
pub enum LoiDongHo {
    /// RTC khong tra loi tren bus
    KhongTraLoi,
    /// Dao dong tung bi dung (mat nguon pin): gio khong con dung, can `dat()` lai
    MatGio,
}

pub struct DongHo {
    i2c: I2c<'static, Blocking, Master>,
}

impl DongHo {
    pub(crate) fn new(i2c: I2c<'static, Blocking, Master>) -> Self {
        DongHo { i2c }
    }

    /// Doc gio hien tai.
    pub fn doc(&mut self) -> Result<Gio, LoiDongHo> {
        let mut b = [0u8; 3];
        self.i2c.blocking_write_read(DIA_CHI, &[0x04], &mut b).map_err(|_| LoiDongHo::KhongTraLoi)?;
        if b[0] & 0x80 != 0 {
            return Err(LoiDongHo::MatGio);
        }
        Ok(Gio { giay: tu_bcd(b[0] & 0x7F), phut: tu_bcd(b[1] & 0x7F), gio: tu_bcd(b[2] & 0x3F) })
    }

    /// Dat gio. Ghi thanh ghi giay cung xoa co "mat gio".
    pub fn dat(&mut self, g: Gio) -> Result<(), LoiDongHo> {
        let lenh = [0x04, sang_bcd(g.giay), sang_bcd(g.phut), sang_bcd(g.gio)];
        self.i2c.blocking_write(DIA_CHI, &lenh).map_err(|_| LoiDongHo::KhongTraLoi)
    }

    /// Dia chi nao tra loi tren I2C1 (chan doan).
    pub fn quet(&mut self, mut bao: impl FnMut(u8)) {
        for dia_chi in 0x08..0x78u8 {
            let mut b = [0u8; 1];
            if self.i2c.blocking_read(dia_chi, &mut b).is_ok() {
                bao(dia_chi);
            }
        }
    }
}

fn tu_bcd(b: u8) -> u8 {
    (b >> 4) * 10 + (b & 0x0F)
}

fn sang_bcd(n: u8) -> u8 {
    ((n / 10) << 4) | (n % 10)
}
