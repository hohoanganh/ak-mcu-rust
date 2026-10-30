//! Bo mach AK MCU KIT cho cac bai demo Rust.
//!
//! Gom phan khoi tao phan cung de `main.rs` cua moi bai chi con dung y can day:
//!
//! ```ignore
//! let kit = ak_kit::init();
//! in_ra!("Xin chao!");
//! kit.led.dao();
//! ```
//!
//! - Xung nhip 32 MHz (PLL tu HSI 16 MHz x6 / 3), nhu firmware C.
//! - LED PB8: noi giua +3V3 va chan, sang khi chan o muc thap — `Led` giau chuyen nay di.
//! - Cong COM: USART1 (PA9 TX, PA10 RX), 115200 8N1, qua cong COM ao cua ST-Link.
//! - Coi: PB0 = TIM3_CH3 (`Coi::keu(tan_so, ms).await`).
//! - Man hinh: OLED SSD1309 128x64, ve bang `embedded-graphics` (`ManHinh`).
//! - Nut: ▼ PB3, ▲ PC13, ⏎ PB4 (trai sang phai), `nut.cho_bam().await` bang ngat EXTI.
//! - Panic: in thong diep kem file:dong ra cong COM roi nhay LED nhanh mai mai.

#![no_std]

mod coi;
mod man_hinh;
mod nut;

pub use coi::Coi;
pub use man_hinh::ManHinh;
pub use nut::Nut;

use core::cell::RefCell;
use core::fmt::Write;
use core::panic::PanicInfo;

use critical_section::Mutex;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Flex, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_stm32::mode::Blocking;
use embassy_stm32::time::hz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usart::{self, UartTx};

/// Phan cung cua kit da khoi tao xong.
pub struct Kit {
    pub led: Led,
    pub coi: Coi,
    pub man_hinh: ManHinh,
    /// Nut trai, in hinh ▼ (PB3)
    pub nut_xuong: Nut,
    /// Nut giua, in hinh ▲ (PC13) — giu nut nay khi bam RESET de vao che do nap
    pub nut_len: Nut,
    /// Nut phai, in hinh ⏎ (PB4)
    pub nut_ok: Nut,
}

bind_interrupts!(struct NgatNut {
    EXTI3 => exti::InterruptHandler<interrupt::typelevel::EXTI3>;
    EXTI4 => exti::InterruptHandler<interrupt::typelevel::EXTI4>;
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

/// LED tren kit (PB8).
pub struct Led {
    chan: Output<'static>,
}

impl Led {
    pub fn bat(&mut self) {
        self.chan.set_low();
    }

    pub fn tat(&mut self) {
        self.chan.set_high();
    }

    pub fn dao(&mut self) {
        self.chan.toggle();
    }
}

static CONG_COM: Mutex<RefCell<Option<UartTx<'static, Blocking>>>> = Mutex::new(RefCell::new(None));

/// Khoi tao kit: xung nhip, LED (tat), cong COM. Goi mot lan, o dau `main`.
///
/// Giu nut giua (PC13) trong luc bam RESET: chuong trinh dung o `che_do_nap()`,
/// khong chay Embassy, de ST-Link luon ket noi duoc ma nap lai.
pub fn init() -> Kit {
    if nut_giua_dang_giu() {
        che_do_nap();
    }

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            mul: PllMul::MUL6, // 16 MHz x 6 = 96 MHz
            div: PllDiv::DIV3, // / 3 = 32 MHz
        });
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);

    let led = Led { chan: Output::new(p.PB8, Level::High, Speed::Low) };

    let tx = UartTx::new_blocking(p.USART1, p.PA9, usart::Config::default()).unwrap();
    critical_section::with(|cs| CONG_COM.borrow(cs).replace(Some(tx)));

    let pwm = SimplePwm::new(
        p.TIM3,
        None,
        None,
        Some(PwmPin::new(p.PB0, OutputType::PushPull)),
        None,
        hz(2000),
        CountingMode::EdgeAlignedUp,
    );
    let coi = Coi::new(pwm);

    // Reset man hinh: PA15 cao - thap - cao, roi moi gui lenh
    let mut res = Output::new(p.PA15, Level::High, Speed::Low);
    cho_ms(5);
    res.set_low();
    cho_ms(5);
    res.set_high();
    cho_ms(10);
    core::mem::forget(res); // giu PA15 o muc cao mai mai
    let mut man_hinh = ManHinh::new(Flex::new(p.PB13), Flex::new(p.PB12));
    if !man_hinh.khoi_dong() {
        _in_ra(format_args!("canh bao: man hinh OLED khong tra loi tren I2C (PB13/PB12, 0x3C)"));
        for dia_chi in 0x08..0x78u8 {
            if man_hinh.thu_dia_chi(dia_chi) {
                _in_ra(format_args!("  thiet bi tra loi o dia chi 0x{:02X}", dia_chi));
            }
        }
    }

    let nut_xuong = Nut::new(ExtiInput::new(p.PB3, p.EXTI3, Pull::Up, NgatNut));
    let nut_len = Nut::new(ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, NgatNut));
    let nut_ok = Nut::new(ExtiInput::new(p.PB4, p.EXTI4, Pull::Up, NgatNut));

    Kit { led, coi, man_hinh, nut_xuong, nut_len, nut_ok }
}

// Cho ban (chi dung luc khoi tao, truoc khi co task nao chay). 32 MHz.
fn cho_ms(ms: u32) {
    cortex_m::asm::delay(ms * 32_000);
}

// Doc nut giua ngay sau reset, truoc khi Embassy dung toi phan cung.
// Ghi thang thanh ghi vi chua co gi duoc khoi tao.
fn nut_giua_dang_giu() -> bool {
    const RCC_AHBENR: *mut u32 = 0x4002_381C as *mut u32;
    const GPIOC_PUPDR: *mut u32 = 0x4002_080C as *mut u32;
    const GPIOC_IDR: *const u32 = 0x4002_0810 as *const u32;
    unsafe {
        RCC_AHBENR.write_volatile(RCC_AHBENR.read_volatile() | (1 << 2)); // GPIOC
        let pupdr = GPIOC_PUPDR.read_volatile() & !(0b11 << 26);
        GPIOC_PUPDR.write_volatile(pupdr | (0b01 << 26)); // PC13 keo len
        cortex_m::asm::delay(20_000); // cho dien ap chan on dinh (~10 ms o MSI 2 MHz)
        GPIOC_IDR.read_volatile() & (1 << 13) == 0 // nut noi GND: bam = muc thap
    }
}

// Che do nap: nhay LED cham, khong ngu, khong doi xung nhip. ST-Link vao duoc.
fn che_do_nap() -> ! {
    const RCC_AHBENR: *mut u32 = 0x4002_381C as *mut u32;
    const GPIOB_MODER: *mut u32 = 0x4002_0400 as *mut u32;
    const GPIOB_BSRR: *mut u32 = 0x4002_0418 as *mut u32;
    unsafe {
        RCC_AHBENR.write_volatile(RCC_AHBENR.read_volatile() | (1 << 1)); // GPIOB
        let moder = GPIOB_MODER.read_volatile() & !(0b11 << 16);
        GPIOB_MODER.write_volatile(moder | (0b01 << 16)); // PB8 ra
        loop {
            GPIOB_BSRR.write_volatile(1 << (8 + 16)); // bat
            cortex_m::asm::delay(200_000); // ~100 ms
            GPIOB_BSRR.write_volatile(1 << 8); // tat
            cortex_m::asm::delay(1_800_000); // ~900 ms
        }
    }
}

#[doc(hidden)]
pub fn _in_ra(args: core::fmt::Arguments) {
    critical_section::with(|cs| {
        if let Some(tx) = CONG_COM.borrow(cs).borrow_mut().as_mut() {
            let mut w = GhiCongCom(tx);
            let _ = w.write_fmt(args);
            let _ = w.write_str("\r\n");
        }
    });
}

struct GhiCongCom<'a>(&'a mut UartTx<'static, Blocking>);

impl Write for GhiCongCom<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.blocking_write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

/// In mot dong ra cong COM (115200 8N1), giong `println!`.
#[macro_export]
macro_rules! in_ra {
    ($($arg:tt)*) => {
        $crate::_in_ra(format_args!($($arg)*))
    };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    match info.location() {
        Some(noi) => _in_ra(format_args!(
            "\r\npanic: {} tai {}:{}",
            info.message(),
            noi.file(),
            noi.line()
        )),
        None => _in_ra(format_args!("\r\npanic: {}", info.message())),
    }
    // Nhay LED nhanh de nhin la biet chuong trinh da dung. Ghi thang thanh ghi
    // GPIOB vi luc nay khong con muon dung lai doi tuong Led da trao cho task.
    const GPIOB_BSRR: *mut u32 = 0x4002_0418 as *mut u32;
    loop {
        unsafe { GPIOB_BSRR.write_volatile(1 << (8 + 16)) }; // bat
        cortex_m::asm::delay(3_200_000); // ~100 ms o 32 MHz
        unsafe { GPIOB_BSRR.write_volatile(1 << 8) }; // tat
        cortex_m::asm::delay(3_200_000);
    }
}
