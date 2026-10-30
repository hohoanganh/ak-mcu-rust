//! Man hinh OLED SSD1309 128x64 cua kit.
//!
//! I2C dia chi 0x3C tren PB13 (SCL) / PB12 (SDA): hai chan nay khong thuoc khoi I2C
//! phan cung nen I2C duoc gia lap bang GPIO (open drain, keo len trong), giong
//! firmware C (`kit.c` cua ak-mcu-base). RES o PA15.
//!
//! Ve vao bo dem khung 1 KB bang `embedded-graphics`, roi `cap_nhat()` gui ra man hinh
//! nhung doan cot da doi (ca khung: khoang 40 ms; doi vai chu so: vai ms).

use embassy_stm32::gpio::{Flex, Pull, Speed};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

const DIA_CHI: u8 = 0x3C;
pub const RONG: usize = 128;
pub const CAO: usize = 64;

/// Chuoi khoi tao cua U8g2 cho SSD1309 128x64, che do dia chi theo trang —
/// y het `lcd_init_seq` trong `kit.c`.
const KHOI_TAO: &[u8] = &[
    0xAE, // tat man hinh
    0xD5, 0xA0, // dao dong noi
    0x40, // dong bat dau 0
    0x20, 0x02, // dia chi theo trang
    0xA1, // dao cot
    0xC8, // dao chieu quet
    0xDA, 0x12, // noi chan COM
    0x81, 0x6F, // do tuong phan
    0xD9, 0xD3, // nap truoc
    0xDB, 0x20, // VCOMH
    0x2E, // tat cuon
    0xA4, // hien RAM
    0xA6, // khong dao mau
    0xAF, // bat man hinh
    0xB0, 0x00, 0x10, // ve trang 0, cot 0
];

pub struct ManHinh {
    scl: Flex<'static>,
    sda: Flex<'static>,
    khung: [u8; RONG * CAO / 8],
    /// Ban sao khung da gui: `cap_nhat()` chi gui doan cot da doi cua moi trang.
    da_gui: [u8; RONG * CAO / 8],
}

impl ManHinh {
    pub(crate) fn new(scl: Flex<'static>, sda: Flex<'static>) -> Self {
        let mut mh = ManHinh { scl, sda, khung: [0; RONG * CAO / 8], da_gui: [0xFF; RONG * CAO / 8] };
        for chan in [&mut mh.scl, &mut mh.sda] {
            chan.set_high(); // tha day truoc khi thanh dau ra
            chan.set_as_input_output_pull(Speed::Low, Pull::Up);
        }
        mh
    }

    /// Gui chuoi khoi tao. Tra ve false neu man hinh khong tra loi tren bus.
    pub(crate) fn khoi_dong(&mut self) -> bool {
        let ok = self.gui_lenh(KHOI_TAO);
        self.xoa();
        ok && self.cap_nhat()
    }

    /// Xoa bo dem khung (chua gui ra man hinh).
    pub fn xoa(&mut self) {
        self.khung.fill(0);
    }

    /// Gui ra man hinh nhung gi da doi tu lan truoc. Tra ve false neu man hinh khong tra loi.
    pub fn cap_nhat(&mut self) -> bool {
        let mut ok = true;
        for trang in 0..(CAO / 8) {
            let dong = trang * RONG..(trang + 1) * RONG;
            let moi = &self.khung[dong.clone()];
            let cu = &self.da_gui[dong.clone()];
            let Some(dau) = (0..RONG).find(|&x| moi[x] != cu[x]) else {
                continue; // trang nay khong doi
            };
            let cuoi = (0..RONG).rfind(|&x| moi[x] != cu[x]).unwrap();
            ok &= self.gui_lenh(&[0xB0 | trang as u8, (dau & 0x0F) as u8, 0x10 | (dau >> 4) as u8]);
            self.bat_dau();
            ok &= self.gui_byte(DIA_CHI << 1);
            ok &= self.gui_byte(0x40); // byte dieu khien: du lieu theo sau
            for x in dau..=cuoi {
                ok &= self.gui_byte(self.khung[trang * RONG + x]);
            }
            self.ket_thuc();
            self.da_gui[trang * RONG + dau..=trang * RONG + cuoi]
                .copy_from_slice(&self.khung[trang * RONG + dau..=trang * RONG + cuoi]);
        }
        ok
    }

    /// Co thiet bi nao tra ACK o dia chi 7 bit `dia_chi` tren bus man hinh khong?
    pub(crate) fn thu_dia_chi(&mut self, dia_chi: u8) -> bool {
        self.bat_dau();
        let ack = self.gui_byte(dia_chi << 1);
        self.ket_thuc();
        ack
    }

    fn gui_lenh(&mut self, lenh: &[u8]) -> bool {
        self.bat_dau();
        let mut ok = self.gui_byte(DIA_CHI << 1);
        ok &= self.gui_byte(0x00); // byte dieu khien: lenh theo sau
        for &b in lenh {
            ok &= self.gui_byte(b);
        }
        self.ket_thuc();
        ok
    }

    // --- I2C gia lap: open drain, "1" = tha day cho dien tro keo len ---

    fn nghi() {
        cortex_m::asm::delay(8);
    }

    // Doi SDA roi nghi truoc khi keo SCL len. Day SDA cua man hinh chi co dien tro keo
    // len yeu nen len muc cao cham: o 32 MHz, keo SCL ngay lenh ke tiep thi man hinh doc
    // sai bit va khong tra ACK (o xung nhip thap CPU tu cham nen loi khong lo ra).
    fn dat_sda(&mut self, cao: bool) {
        if cao {
            self.sda.set_high();
        } else {
            self.sda.set_low();
        }
        Self::nghi();
    }

    fn dat_scl(&mut self, cao: bool) {
        if cao {
            self.scl.set_high();
        } else {
            self.scl.set_low();
        }
        Self::nghi();
    }

    fn bat_dau(&mut self) {
        self.dat_sda(true);
        self.dat_scl(true);
        self.dat_sda(false);
        self.dat_scl(false);
    }

    fn ket_thuc(&mut self) {
        self.dat_sda(false);
        self.dat_scl(true);
        self.dat_sda(true);
    }

    /// Gui mot byte, tra ve true neu thiet bi tra ACK.
    fn gui_byte(&mut self, mut b: u8) -> bool {
        for _ in 0..8 {
            self.dat_sda(b & 0x80 != 0);
            b <<= 1;
            self.dat_scl(true);
            self.dat_scl(false);
        }
        self.dat_sda(true);
        self.dat_scl(true);
        let ack = self.sda.is_low();
        self.dat_scl(false);
        ack
    }
}

impl OriginDimensions for ManHinh {
    fn size(&self) -> Size {
        Size::new(RONG as u32, CAO as u32)
    }
}

impl DrawTarget for ManHinh {
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
                self.khung[i] |= bit;
            } else {
                self.khung[i] &= !bit;
            }
        }
        Ok(())
    }
}

/// Ve man hinh panic: chuong trinh da dung, lay lai chan man hinh bang `steal()`.
/// `thong_diep`: ly do; `tep`, `dong`: noi xay ra.
pub(crate) fn hien_panic(thong_diep: &str, tep: &str, dong: u32) {
    use core::fmt::Write;
    use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
    use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
    use embedded_graphics::text::Text;

    let p = unsafe { embassy_stm32::Peripherals::steal() };
    let mut mh = ManHinh::new(Flex::new(p.PB13), Flex::new(p.PB12));
    let chu_nho = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Dai den chu trang "PANIC" o tren cung
    Rectangle::new(Point::zero(), Size::new(RONG as u32, 20))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut mh)
        .ok();
    Text::new("PANIC", Point::new(39, 15), MonoTextStyle::new(&FONT_10X20, BinaryColor::Off)).draw(&mut mh).ok();

    // Noi xay ra: chi ten tep (bo thu muc) + so dong
    let ten_tep = tep.rsplit(['/', '\\']).next().unwrap_or(tep);
    let mut noi: heapless::String<32> = heapless::String::new();
    let _ = write!(noi, "{} dong {}", ten_tep, dong);
    Text::new(&noi, Point::new(0, 30), chu_nho).draw(&mut mh).ok();

    // Ly do, ngat dong moi 21 ky tu, toi da 3 dong
    let byte = thong_diep.as_bytes();
    for (i, khuc) in byte.chunks(21).take(3).enumerate() {
        if let Ok(s) = core::str::from_utf8(khuc) {
            Text::new(s, Point::new(0, 42 + 10 * i as i32), chu_nho).draw(&mut mh).ok();
        }
    }
    mh.cap_nhat();
}
