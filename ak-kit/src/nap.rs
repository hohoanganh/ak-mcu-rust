//! Nghe lenh nap tren cong COM (giao thuc `fw_proto` cua ak-mcu-base), de cap nhat
//! bai Rust bang `tools/ak_fw.py` qua cong COM nhu app C — khong can ST-Link.
//!
//! Ho tro: INFO (vai tro app, phien ban doc tu header anh), LOADER (vao bootloader de
//! nhan anh moi), RESET, CONFIRM. Anh moi cai chay "thu": sau 30 s chay on dinh thi tu
//! xac nhan voi bootloader, khong thi sau 3 lan khoi dong bootloader cai lai anh cu.

use ak_boot::boot_ctrl::{self, LoiNvm, Nvm};
use ak_boot::khung::{self, BoTach, Khung};
use core::cell::RefCell;
use critical_section::Mutex;
use embassy_stm32::flash::{self, Flash};
use embassy_stm32::interrupt::typelevel::{Handler, Interrupt, USART1};
use embassy_stm32::pac;
use embassy_time::{Duration, Instant, Timer};

const HEADER_ANH: u32 = 0x0800_3000; // header 256 B cua anh dang chay
const XAC_NHAN_SAU: Duration = Duration::from_secs(30);

/// EEPROM cua chip lam NVM cho `boot_ctrl` (offset 0 = 0x0808_0000).
pub(crate) struct Eeprom(pub(crate) Flash<'static, flash::Blocking>);

impl Nvm for Eeprom {
    fn doc(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), LoiNvm> {
        self.0.eeprom_read_slice(offset, buf).map_err(|_| LoiNvm::Doc)
    }
    fn ghi(&mut self, offset: u32, du_lieu: &[u8]) -> Result<(), LoiNvm> {
        self.0.eeprom_write_slice(offset, du_lieu).map_err(|_| LoiNvm::Ghi)
    }
}

/// Byte nhan tu cong COM. USART cua L151 chi giu 1 byte: neu cho task doc thang thanh
/// ghi, mot task khac ban (ve OLED mat toi 25 ms ~ 290 byte) la mat byte cua khung lenh.
/// Ngat RXNE day tung byte vao day, task lay ra luc ranh.
static NHAN: Mutex<RefCell<heapless::Deque<u8, 256>>> = Mutex::new(RefCell::new(heapless::Deque::new()));

/// Ngat USART1: chi nhan byte (phia gui van ghi chan, khong dung ngat).
pub struct NgatNhan;

impl Handler<USART1> for NgatNhan {
    unsafe fn on_interrupt() {
        let usart = pac::USART1;
        let sr = usart.sr().read();
        if sr.rxne() || sr.ore() {
            let b = usart.dr().read().dr() as u8; // doc DR xoa ca RXNE va ORE
            critical_section::with(|cs| {
                NHAN.borrow(cs).borrow_mut().push_back(b).ok();
            });
        }
    }
}

/// Bat ngat nhan cua USART1 (USART da duoc `Uart::new_blocking` cau hinh, RX dang bat).
pub(crate) fn bat_ngat_nhan() {
    pac::USART1.cr1().modify(|w| w.set_rxneie(true));
    USART1::unpend();
    unsafe { USART1::enable() };
}

fn lay_byte() -> Option<u8> {
    critical_section::with(|cs| NHAN.borrow(cs).borrow_mut().pop_front())
}

#[embassy_executor::task]
pub(crate) async fn nghe_lenh_nap(mut nvm: Eeprom) {
    let mut bo_tach = BoTach::new();
    let mut da_xac_nhan = false;

    loop {
        match lay_byte() {
            Some(b) => {
                let bay_gio = Instant::now().as_millis() as u32;
                if let Some(k) = bo_tach.nap(b, bay_gio) {
                    xu_ly(&k, &mut nvm).await;
                }
            }
            None => Timer::after_millis(1).await, // chua co byte: nhuong luot
        }
        if !da_xac_nhan && Instant::now().as_ticks() >= XAC_NHAN_SAU.as_ticks() {
            da_xac_nhan = true;
            if let Ok(true) = boot_ctrl::xac_nhan(&mut nvm) {
                crate::_in_ra(format_args!("anh moi chay on dinh 30 s: da xac nhan voi bootloader"));
            }
        }
    }
}

async fn xu_ly(k: &Khung, nvm: &mut Eeprom) {
    match k.cmd {
        khung::CMD_INFO => tra_loi(k, &info()),
        khung::CMD_CONFIRM => match boot_ctrl::xac_nhan(nvm) {
            Ok(dang_thu) => tra_loi(k, &[0, dang_thu as u8]),
            Err(_) => tra_loi(k, &[khung::LOI_FLASH, 0]),
        },
        khung::CMD_LOADER => {
            if boot_ctrl::dat_lenh(nvm, boot_ctrl::CMD_LOADER).is_err() {
                tra_loi(k, &[khung::LOI_FLASH]);
                return;
            }
            tra_loi(k, &[0]);
            crate::_in_ra(format_args!("khoi dong lai vao bootloader de nhan anh moi"));
            khoi_dong_lai().await;
        }
        khung::CMD_RESET => {
            tra_loi(k, &[0]);
            khoi_dong_lai().await;
        }
        _ => tra_loi(k, &[khung::LOI_CMD]),
    }
}

/// Tra loi INFO 38 B, giong `fw_proto.c`: ma loi, proto, vai tro, phien ban (8),
/// ten board (16), staging (4), max_chunk (2), trang thai (1), da ghi (4).
fn info() -> [u8; 38] {
    let mut r = [0u8; 38];
    r[1] = 1; // FW_PROTO_VERSION
    r[2] = 1; // vai tro: app
    let hdr = unsafe { core::slice::from_raw_parts(HEADER_ANH as *const u8, 44) };
    r[3..11].copy_from_slice(&hdr[20..28]); // major minor patch 0 build(4)
    r[11..27].copy_from_slice(&hdr[28..44]); // ten board
    // staging = 0: bai Rust khong nhan anh khi dang chay; ak_fw.py phai `loader` truoc
    r[31] = 128; // max_chunk
    r
}

fn tra_loi(k: &Khung, du_lieu: &[u8]) {
    let mut ra = [0u8; 48];
    let n = khung::dong_goi(k.cmd | khung::TRA_LOI, k.seq, du_lieu, &mut ra);
    crate::_gui_byte(&ra[..n]);
}

async fn khoi_dong_lai() -> ! {
    Timer::after_millis(50).await; // cho cau tra loi ra het day
    cortex_m::peripheral::SCB::sys_reset()
}
