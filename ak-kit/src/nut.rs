//! Ba nut bam duoi man hinh, noi GND, keo len trong. Bam = muc thap.
//!
//! Dung ngat EXTI: `cho_bam().await` khong hoi trang thai chan lien tuc,
//! ngat danh thuc dung task dang cho.

use embassy_stm32::exti::ExtiInput;
use embassy_stm32::mode::Async;
use embassy_time::Timer;

pub struct Nut {
    chan: ExtiInput<'static, Async>,
}

impl Nut {
    pub(crate) fn new(chan: ExtiInput<'static, Async>) -> Self {
        Nut { chan }
    }

    /// Cho toi khi nut duoc bam. Loc doi phim 20 ms: canh xuong luc tha nut
    /// (do tiep diem doi) khong bi tinh la mot lan bam.
    pub async fn cho_bam(&mut self) {
        loop {
            self.chan.wait_for_falling_edge().await;
            Timer::after_millis(20).await;
            if self.chan.is_low() {
                return;
            }
        }
    }

    /// Nut co dang duoc giu khong.
    pub fn dang_bam(&self) -> bool {
        self.chan.is_low()
    }
}
