//! Man hinh cua bai 2, 3, 4, 5 va man PANIC.

use core::fmt::Write;
use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
#[allow(unused_imports)] // tren chip can cho sin/cos; tren may tinh f32 co san
use micromath::F32Ext;

fn chu_nho() -> MonoTextStyle<'static, BinaryColor> {
    MonoTextStyle::new(&FONT_6X10, BinaryColor::On)
}

fn chu_to() -> MonoTextStyle<'static, BinaryColor> {
    MonoTextStyle::new(&FONT_10X20, BinaryColor::On)
}

/// Bai 2: thoi gian chay (ms) va nhip cua ba viec.
pub fn ve_ba_viec<D: DrawTarget<Color = BinaryColor>>(d: &mut D, ms: u64) {
    let mut so: heapless::String<16> = heapless::String::new();
    write!(so, "{}.{} s", ms / 1000, (ms / 100) % 10).unwrap();

    Text::new("3 viec cung luc", Point::new(19, 10), chu_nho()).draw(d).ok();
    Text::new(&so, Point::new(24, 38), chu_to()).draw(d).ok();
    Text::new("LED 300ms  COI 1s", Point::new(13, 52), chu_nho()).draw(d).ok();
    Text::new("OLED 100ms", Point::new(34, 62), chu_nho()).draw(d).ok();
}

/// Bai 3: nut vua bam ("DO", "MI", "SOL" hoac "BAM NUT" luc dau) va so lan bam.
pub fn ve_nut_bam<D: DrawTarget<Color = BinaryColor>>(d: &mut D, ten: &str, so_lan: u32) {
    let mut dem: heapless::String<20> = heapless::String::new();
    write!(dem, "da bam {} lan", so_lan).unwrap();

    Text::new("Dan 3 phim", Point::new(34, 10), chu_nho()).draw(d).ok();
    let x = 64 - (ten.len() as i32 * 10) / 2; // canh giua
    Text::new(ten, Point::new(x, 38), chu_to()).draw(d).ok();
    Text::new(&dem, Point::new(64 - dem.len() as i32 * 3, 58), chu_nho()).draw(d).ok();
}

/// Bai 4: dong ho kim ben trai, gio so ben phai.
pub fn ve_dong_ho<D: DrawTarget<Color = BinaryColor>>(d: &mut D, gio: u8, phut: u8, giay: u8) {
    let net = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let tam = Point::new(31, 32);

    // Mat dong ho va 12 vach
    Circle::with_center(tam, 61).into_styled(net).draw(d).ok();
    for i in 0..12 {
        Line::new(diem(tam, i as f32 / 12.0, 26.0), diem(tam, i as f32 / 12.0, 29.0)).into_styled(net).draw(d).ok();
    }
    // Ba kim: gio (ngan, dam), phut, giay (dai, manh)
    let phut_le = phut as f32 + giay as f32 / 60.0;
    let gio_le = (gio % 12) as f32 + phut_le / 60.0;
    let dam = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    let vua = PrimitiveStyle::with_stroke(BinaryColor::On, 2);
    Line::new(tam, diem(tam, gio_le / 12.0, 15.0)).into_styled(dam).draw(d).ok();
    Line::new(tam, diem(tam, phut_le / 60.0, 22.0)).into_styled(vua).draw(d).ok();
    Line::new(tam, diem(tam, giay as f32 / 60.0, 27.0)).into_styled(net).draw(d).ok();

    // Gio so ben phai
    let mut chu: heapless::String<8> = heapless::String::new();
    write!(chu, "{:02}:{:02}", gio, phut).unwrap();
    Text::new(&chu, Point::new(68, 30), chu_to()).draw(d).ok();
    chu.clear();
    write!(chu, ":{:02}", giay).unwrap();
    Text::new(&chu, Point::new(98, 46), chu_nho()).draw(d).ok();
}

/// Diem tren vong tron ban kinh `r`, o vi tri `phan` cua mot vong (0 = 12 gio, quay theo chieu kim).
fn diem(tam: Point, phan: f32, r: f32) -> Point {
    let goc = phan * 2.0 * core::f32::consts::PI;
    Point::new(tam.x + (goc.sin() * r) as i32, tam.y - (goc.cos() * r) as i32)
}

/// Bai 5: mang 5 phan tu, chi so dang chon, gia tri vua doc (neu co).
pub fn ve_an_toan<D: DrawTarget<Color = BinaryColor>>(d: &mut D, chi_so: usize, do_dai: usize, vua_doc: Option<i32>) {
    let mut dong: heapless::String<24> = heapless::String::new();

    Text::new("mang co 5 phan tu", Point::new(13, 9), chu_nho()).draw(d).ok();
    match vua_doc {
        Some(v) => write!(dong, "diem[{}]={}", chi_so, v),
        None => write!(dong, "diem[{}]", chi_so),
    }
    .unwrap();
    Text::new(&dong, Point::new(64 - dong.len() as i32 * 5, 34), chu_to()).draw(d).ok();
    let goi_y = if chi_so < do_dai { "v ^ chon   OK doc" } else { "NGOAI MANG! bam OK?" };
    Text::new(goi_y, Point::new(64 - goi_y.len() as i32 * 3, 56), chu_nho()).draw(d).ok();
}

/// Man hinh panic: dai trang chu den "PANIC", noi xay ra (ten tep + dong), ly do toi da 3 dong.
pub fn ve_panic<D: DrawTarget<Color = BinaryColor>>(d: &mut D, thong_diep: &str, tep: &str, dong: u32) {
    Rectangle::new(Point::zero(), Size::new(128, 20))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(d)
        .ok();
    Text::new("PANIC", Point::new(39, 15), MonoTextStyle::new(&FONT_10X20, BinaryColor::Off)).draw(d).ok();

    // Noi xay ra: chi ten tep (bo thu muc) + so dong
    let ten_tep = tep.rsplit(['/', '\\']).next().unwrap_or(tep);
    let mut noi: heapless::String<32> = heapless::String::new();
    let _ = write!(noi, "{} dong {}", ten_tep, dong);
    Text::new(&noi, Point::new(0, 30), chu_nho()).draw(d).ok();

    // Ly do, ngat dong moi 21 ky tu, toi da 3 dong
    for (i, khuc) in thong_diep.as_bytes().chunks(21).take(3).enumerate() {
        if let Ok(s) = core::str::from_utf8(khuc) {
            Text::new(s, Point::new(0, 42 + 10 * i as i32), chu_nho()).draw(d).ok();
        }
    }
}
