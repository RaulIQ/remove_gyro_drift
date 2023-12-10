#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_stm32::{self, into_ref, Peripheral, PeripheralRef};
use embassy_stm32::gpio::OutputType::PushPull;
use embassy_stm32::time::{Hertz, hz};
use embassy_stm32::timer::{Channel, Channel1Pin, OutputCompareMode, CaptureCompare16bitInstance, Channel2Pin, Channel3Pin, Channel4Pin, CaptureCompare32bitInstance};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {

    let p = embassy_stm32::init(Default::default());

    // let mut pwm = SimplePwm16::new(p.TIM3, Some(p.PA6), hz(440));
    let mut pwm = SimplePwm16::new(p.TIM5, p.PA0, hz(440));
    pwm.set_duty(Channel::Ch1, pwm.get_max_duty() / 2);
    pwm.enable(Channel::Ch1);
    loop {

    }
}

struct SimplePwm16<'d, T> {
    inner: PeripheralRef<'d, T>
}

impl<'d, T: CaptureCompare16bitInstance> SimplePwm16<'d, T> {
    fn new(
        tim: impl Peripheral<P = T> + 'd,
        ch: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        freq: Hertz
    ) -> Self {

        into_ref!(tim, ch);

        ch.set_as_af(ch.af_num(), PushPull.into());

        T::enable_and_reset();

        let mut this = Self {inner: tim};

        this.set_freq(freq);
        this.inner.start();

        let r = T::regs_gp16();
        r.ccmr_output(0)
            .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));

        this
    }

    fn enable(&mut self, channel: Channel) {
        T::regs_gp16().ccer().modify(|w| w.set_cce(channel.raw(), true));
    }

    fn disable(&mut self, channel: Channel) {
        T::regs_gp16().ccer().modify(|w| w.set_cce(channel.raw(), false));
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        self.inner.set_frequency(freq);
    }

    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }

    fn set_duty(&mut self, channel: Channel, duty: u16) {
        T::regs_gp16().ccr(channel.raw()).modify(|w| w.set_ccr(duty))
    }
}

struct SimplePwm32<'d, T: CaptureCompare32bitInstance> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare32bitInstance> SimplePwm32<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        ch: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        freq: Hertz,
    ) -> Self {
        into_ref!(tim, ch);

        T::enable_and_reset();

        ch.set_as_af(ch.af_num(), PushPull.into());
        let mut this = Self { inner: tim };

        this.set_freq(freq);
        this.inner.start();

        let r = T::regs_gp32();
        r.ccmr_output(0)
            .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));

        this
    }

    pub fn enable(&mut self, channel: Channel) {
        T::regs_gp32().ccer().modify(|w| w.set_cce(channel.raw(), true));
    }

    pub fn disable(&mut self, channel: Channel) {
        T::regs_gp32().ccer().modify(|w| w.set_cce(channel.raw(), false));
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        <T as embassy_stm32::timer::low_level::GeneralPurpose32bitInstance>::set_frequency(&mut self.inner, freq);
    }

    pub fn get_max_duty(&self) -> u32 {
        T::regs_gp32().arr().read().arr()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        T::regs_gp32().ccr(channel.raw()).modify(|w| w.set_ccr(duty))
    }
}











