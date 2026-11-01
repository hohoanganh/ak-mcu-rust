//! Su co cua lan chay truoc, giong ak-mcu-base: luc loi (panic, HardFault) chi ghi vai
//! su kien vao RAM `.noinit` (con nguyen qua reset), lan khoi dong sau moi chep vao nhat ky
//! su co trong EEPROM (`ak_boot::su_co`, cung dinh dang voi `crash_log.c`). Nap lai app C
//! roi go `crash` cung doc duoc.
//!
//! Watchdog 8 s nhu app C: task `canh_gac` vuot no moi giay. Mot task chay vong lap khong
//! `.await` thi bo chay task khong toi luot `canh_gac` -> watchdog reset chip -> lan sau
//! nhat ky ghi "watchdog".

use ak_boot::boot_ctrl::Nvm;
use ak_boot::su_co::{self, BanGhi};
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;

/// `port_noinit_t` cua ak-mcu-base (`port/stm32l151/port_stm32.h`), dau RAM, 52 B.
/// Bootloader va app C dung chung; `memory.x` cua bai Rust bat dau RAM sau no.
const NOINIT: usize = 0x2000_0000;
const REASON_MAGIC: usize = 8;
const RESET_REASON: usize = 12;
const FAULT_MAGIC: usize = 16;
const FAULT_PC: usize = 20;
const FAULT_LR: usize = 24;
const FAULT_CFSR: usize = 28;
const FATAL_MAGIC: usize = 32;
const FATAL_TAG: usize = 36;
const FATAL_CODE: usize = 40;

const PORT_REASON_MAGIC: u32 = 0x5253_5421;
const PORT_FAULT_MAGIC: u32 = 0x4641_5521;
const PORT_FATAL_MAGIC: u32 = 0x4654_4C21;
/// `HAL_RESET_REASON_WATCHDOG`
const RESET_WATCHDOG: u32 = 4;

/// Watchdog nhu `APP_WDT_TIMEOUT_MS` cua app C.
const WATCHDOG_US: u32 = 8_000_000;
/// Thanh ghi KR cua IWDG: ghi 0xAAAA = vuot (dung trong panic, khi khong con doi tuong).
const IWDG_KR: *mut u32 = 0x4000_3000 as *mut u32;

fn doc(o: usize) -> u32 {
    unsafe { ((NOINIT + o) as *const u32).read_volatile() }
}

fn ghi(o: usize, v: u32) {
    unsafe { ((NOINIT + o) as *mut u32).write_volatile(v) }
}

/// 4 ky tu dau cua ten tep (bo duong dan), dong goi nhu mot u32 little-endian.
fn bon_ky_tu(tep: &str) -> u32 {
    let ten = tep.rsplit(['/', '\\']).next().unwrap_or(tep).as_bytes();
    let mut b = [0u8; 4];
    for (i, c) in ten.iter().take(4).enumerate() {
        b[i] = *c;
    }
    u32::from_le_bytes(b)
}

/// Goi trong panic: ghi su kien nhu `FATAL("RUST", 1)` cua C, kem tep va dong o cho
/// FAULT_PC / FAULT_LR (khong dat FAULT_MAGIC nen app C khong doc nham la HardFault).
pub(crate) fn ghi_panic(tep: &str, dong: u32) {
    ghi(FAULT_PC, dong);
    ghi(FAULT_LR, bon_ky_tu(tep));
    ghi(FATAL_TAG, su_co::THE_RUST);
    ghi(FATAL_CODE, 1);
    ghi(FATAL_MAGIC, PORT_FATAL_MAGIC);
}

/// Goi trong HardFault, giong `HardFault_Handler` cua C.
pub(crate) fn ghi_hardfault(pc: u32, lr: u32) {
    let cfsr = unsafe { (0xE000_ED28 as *const u32).read_volatile() };
    ghi(FAULT_PC, pc);
    ghi(FAULT_LR, lr);
    ghi(FAULT_CFSR, cfsr);
    ghi(FAULT_MAGIC, PORT_FAULT_MAGIC);
}

/// Vuot watchdog bang thanh ghi (panic giu man hinh PANIC thi khong de chip bi reset).
pub(crate) fn vuot_watchdog_tho() {
    unsafe { IWDG_KR.write_volatile(0xAAAA) };
}

/// Lan chay truoc ket thuc bat thuong thi chep vao nhat ky su co, tra ve ban ghi do.
pub(crate) fn ghi_lai(nvm: &mut impl Nvm) -> Option<BanGhi> {
    let mut r = BanGhi { task: su_co::TASK_IDLE, ..Default::default() };
    if doc(FAULT_MAGIC) == PORT_FAULT_MAGIC {
        ghi(FAULT_MAGIC, 0);
        ghi(FATAL_MAGIC, 0);
        r.loai = su_co::LOAI_HARDFAULT;
        r.pc = doc(FAULT_PC);
        r.lr = doc(FAULT_LR);
        r.info = doc(FAULT_CFSR);
    } else if doc(FATAL_MAGIC) == PORT_FATAL_MAGIC {
        ghi(FATAL_MAGIC, 0);
        r.loai = su_co::LOAI_FATAL;
        r.ma = doc(FATAL_CODE) as u8;
        r.info = doc(FATAL_TAG);
        if r.info == su_co::THE_RUST {
            r.pc = doc(FAULT_PC); // dong
            r.lr = doc(FAULT_LR); // 4 ky tu dau cua tep
        }
    } else if doc(REASON_MAGIC) == PORT_REASON_MAGIC && doc(RESET_REASON) == RESET_WATCHDOG {
        r.loai = su_co::LOAI_WATCHDOG;
    } else {
        return None;
    }
    su_co::them(nvm, &mut r).ok();
    Some(r)
}

/// In ban ghi ra cong COM, mot dong.
pub(crate) fn in_ban_ghi(r: &BanGhi) {
    match r.loai {
        su_co::LOAI_HARDFAULT => crate::_in_ra(format_args!(
            "su co lan truoc: hardfault pc 0x{:08X} lr 0x{:08X} cfsr 0x{:08X}",
            r.pc, r.lr, r.info
        )),
        su_co::LOAI_FATAL if r.info == su_co::THE_RUST => {
            let tep = r.lr.to_le_bytes();
            let tep = core::str::from_utf8(&tep).unwrap_or("?").trim_end_matches('\0');
            crate::_in_ra(format_args!("su co lan truoc: panic {}... dong {}", tep, r.pc))
        }
        su_co::LOAI_FATAL => crate::_in_ra(format_args!("su co lan truoc: fatal 0x{:02X}", r.ma)),
        _ => crate::_in_ra(format_args!(
            "su co lan truoc: watchdog (mot task chay qua 8 s khong nhuong luot)"
        )),
    }
}

/// Bat watchdog. Tu day phai co `canh_gac` chay, khong thi 8 s sau chip reset.
pub(crate) fn bat_watchdog(iwdg: embassy_stm32::Peri<'static, IWDG>) -> IndependentWatchdog<'static, IWDG> {
    let mut wdg = IndependentWatchdog::new(iwdg, WATCHDOG_US);
    wdg.unleash();
    wdg
}

/// Vuot watchdog moi giay. Chi chay duoc khi moi task khac deu chiu nhuong luot.
#[embassy_executor::task]
pub(crate) async fn canh_gac(mut wdg: IndependentWatchdog<'static, IWDG>) {
    loop {
        wdg.pet();
        Timer::after_secs(1).await;
    }
}
