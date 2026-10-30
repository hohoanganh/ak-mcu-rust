//! Coi thu dong tren PB0 = TIM3_CH3: phat tieng bang song vuong (PWM 50 %).

use embassy_stm32::peripherals::TIM3;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_time::Timer;

pub struct Coi {
    pwm: SimplePwm<'static, TIM3>,
}

impl Coi {
    pub(crate) fn new(pwm: SimplePwm<'static, TIM3>) -> Self {
        let mut coi = Coi { pwm };
        coi.tat();
        coi
    }

    /// Keu o `tan_so` Hz trong `ms` mili giay. Trong luc keu, viec khac van chay.
    pub async fn keu(&mut self, tan_so: u32, ms: u64) {
        self.pwm.set_frequency(hz(tan_so));
        {
            let mut kenh = self.pwm.ch3();
            kenh.set_duty_cycle_percent(50);
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
