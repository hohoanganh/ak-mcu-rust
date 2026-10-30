//! Coi thu dong tren PB0 = TIM3_CH3: phat tieng bang song vuong (PWM).
//!
//! Do to theo do rong xung: 50 % la to nhat, nho hon thi nho dan.
//! Mac dinh 1 % (nho nhat con nghe duoc) cho do on khi ngoi canh kit;
//! `dat_am_luong(0)` tat han, `dat_am_luong(50)` to nhat (vi du khi quay clip).

use embassy_stm32::peripherals::TIM3;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_time::Timer;

pub struct Coi {
    pwm: SimplePwm<'static, TIM3>,
    am_luong: u8,
}

impl Coi {
    pub(crate) fn new(pwm: SimplePwm<'static, TIM3>) -> Self {
        let mut coi = Coi { pwm, am_luong: 1 };
        coi.tat();
        coi
    }

    /// Am luong 0..50 (do rong xung %): 0 = tat, 50 = to nhat. Mac dinh 1.
    pub fn dat_am_luong(&mut self, phan_tram: u8) {
        self.am_luong = phan_tram.min(50);
    }

    /// Keu o `tan_so` Hz trong `ms` mili giay. Trong luc keu, viec khac van chay.
    pub async fn keu(&mut self, tan_so: u32, ms: u64) {
        if self.am_luong == 0 {
            Timer::after_millis(ms).await; // tat tieng: van giu dung nhip
            return;
        }
        self.pwm.set_frequency(hz(tan_so));
        {
            let mut kenh = self.pwm.ch3();
            kenh.set_duty_cycle_percent(self.am_luong);
            kenh.enable();
        }
        Timer::after_millis(ms).await;
        self.tat();
    }

    /// Im lang.
    pub fn tat(&mut self) {
        let mut kenh = self.pwm.ch3();
        kenh.set_duty_cycle_percent(0);
        kenh.disable();
    }
}
