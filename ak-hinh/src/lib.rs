//! Phan ve man hinh OLED 128x64 cua cac bai demo.
//!
//! Moi ham `ve_*` ve len bat ky `DrawTarget<Color = BinaryColor>` nao: tren kit la `ManHinh`
//! cua `ak-kit` (bo dem khung gui qua I2C), tren may tinh la `KhungHinh` o day. Nho vay cung
//! mot doan ma vua chay tren chip, vua duoc dung thanh GIF (xem `examples/quay.rs`):
//! hinh trong GIF la hinh chuong trinh se ve len kit.
//!
//! Cac ham ve khong xoa man hinh va khong gui ra: ben goi `xoa()` truoc, `cap_nhat()` sau.

#![no_std]

mod ferris;
mod khung;
mod ve;

pub use ferris::{AmThanh, Ferris};
pub use khung::{KhungHinh, CAO, RONG};
pub use ve::{ve_an_toan, ve_ba_viec, ve_dong_ho, ve_nut_bam, ve_panic};
