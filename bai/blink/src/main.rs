//! Nháy LED trên AK MCU KIT bằng Rust, chạy sau bootloader ak-mcu-base.
//!
//! - LED PB8, nối giữa +3V3 và chân: kéo chân xuống thấp thì LED sáng.
//! - Console USART1 (PA9 TX, PA10 RX), 115200 8N1.
//! - Xung nhịp HSI 16 MHz. Bootloader nhảy vào app qua một lần reset, nên chip
//!   tới đây ở trạng thái sạch: đang chạy MSI 2 MHz, watchdog chưa bật.
//!
//! Không dùng thư viện HAL: ghi thẳng thanh ghi theo RM0038, cho dễ đối chiếu
//! với mã C trong ak-mcu-base.

#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use panic_halt as _;

/// Bảng ngắt ngoại vi: STM32L151 loại Cat.1/2 (L151CB) có 45 ngắt (RM0038).
/// 16 + 45 mục = 244 B nên bảng vector căn 256 B là đủ, khớp địa chỉ 0x08003100
/// của bootloader. Mặc định cortex-m-rt tạo 240 mục, đòi căn 1 KB.
/// Chương trình không bật ngắt ngoại vi nào, nên các mục để trống.
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
#[used]
static __INTERRUPTS: [usize; 45] = [0; 45];

const SYSCLK_HZ: u32 = 16_000_000;

const FLASH_ACR: u32 = 0x4002_3C00;
const RCC: u32 = 0x4002_3800;
#[allow(clippy::identity_op)] // giu dang "co so + offset" cho de doi chieu reference manual
const RCC_CR: u32 = RCC + 0x00;
const RCC_CFGR: u32 = RCC + 0x08;
const RCC_AHBENR: u32 = RCC + 0x1C;
const RCC_APB2ENR: u32 = RCC + 0x20;

const GPIOA: u32 = 0x4002_0000;
const GPIOB: u32 = 0x4002_0400;
const MODER: u32 = 0x00;
const BSRR: u32 = 0x18;
const AFRH: u32 = 0x24;

const USART1: u32 = 0x4001_3800;
#[allow(clippy::identity_op)]
const USART_SR: u32 = USART1 + 0x00;
const USART_DR: u32 = USART1 + 0x04;
const USART_BRR: u32 = USART1 + 0x08;
const USART_CR1: u32 = USART1 + 0x0C;

const LED_PIN: u32 = 8;

fn rd(addr: u32) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

fn wr(addr: u32, val: u32) {
    unsafe { write_volatile(addr as *mut u32, val) }
}

fn modify(addr: u32, clear: u32, set: u32) {
    wr(addr, (rd(addr) & !clear) | set);
}

/// MSI 2 MHz -> HSI 16 MHz. Dải điện áp sau reset là range 2 (1,5 V):
/// 16 MHz cần flash 1 wait state, mà muốn có wait state thì phải bật ACC64 trước.
fn clock_init() {
    modify(FLASH_ACR, 0, 1 << 2); // ACC64
    modify(FLASH_ACR, 0, 1 << 0); // LATENCY = 1 WS
    modify(FLASH_ACR, 0, 1 << 1); // PRFTEN
    modify(RCC_CR, 0, 1 << 0); // HSION
    while rd(RCC_CR) & (1 << 1) == 0 {} // HSIRDY
    modify(RCC_CFGR, 0b11, 0b01); // SW = HSI
    while (rd(RCC_CFGR) >> 2) & 0b11 != 0b01 {} // SWS = HSI
}

fn gpio_init() {
    modify(RCC_AHBENR, 0, (1 << 0) | (1 << 1)); // GPIOA, GPIOB
    // PB8 ra (output push-pull), mặc định để mức cao = LED tắt
    wr(GPIOB + BSRR, 1 << LED_PIN);
    modify(GPIOB + MODER, 0b11 << (LED_PIN * 2), 0b01 << (LED_PIN * 2));
}

fn uart_init() {
    modify(RCC_APB2ENR, 0, 1 << 14); // USART1EN
    // PA9, PA10: alternate function 7 (USART1)
    modify(GPIOA + AFRH, (0xF << 4) | (0xF << 8), (7 << 4) | (7 << 8));
    modify(GPIOA + MODER, (0b11 << 18) | (0b11 << 20), (0b10 << 18) | (0b10 << 20));
    wr(USART_BRR, (SYSCLK_HZ + 115_200 / 2) / 115_200); // 139: lệch 0,08 %
    wr(USART_CR1, (1 << 13) | (1 << 3) | (1 << 2)); // UE, TE, RE
}

fn uart_write(s: &str) {
    for b in s.bytes() {
        while rd(USART_SR) & (1 << 7) == 0 {} // TXE
        wr(USART_DR, b as u32);
    }
}

fn uart_write_u32(mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    uart_write(core::str::from_utf8(&buf[i..]).unwrap());
}

fn led(on: bool) {
    // LED nối lên +3V3: bật = kéo chân xuống (BR8), tắt = đẩy chân lên (BS8)
    let bit = if on { 1 << (LED_PIN + 16) } else { 1 << LED_PIN };
    wr(GPIOB + BSRR, bit);
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    clock_init();
    gpio_init();
    uart_init();

    let mut delay = Delay::new(cp.SYST, SYSCLK_HZ);
    uart_write("\r\nXin chao tu Rust! AK MCU KIT, STM32L151, HSI 16 MHz\r\n");

    let mut n: u32 = 0;
    loop {
        led(true);
        delay.delay_ms(200);
        led(false);
        delay.delay_ms(800);
        n = n.wrapping_add(1);
        uart_write("nhay lan ");
        uart_write_u32(n);
        uart_write("\r\n");
    }
}
