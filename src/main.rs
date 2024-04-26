#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::I2c;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::time::hz;
use embassy_time::{Delay, Duration, Timer, Instant};
use nalgebra::{Matrix6x3, Matrix6x4, Matrix1x4, Vector3, RowVector4, Matrix4x3};

use gy91::*;
use {defmt_rtt as _, panic_probe as _};

pub const PI: f32 = core::f32::consts::PI;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

fn find_gyroscope_drift(mpu: &mut Mpu6050<I2c<I2C1>, Delay>) -> Vector3<f32>{
    let mut calibration_values = Vector3::new(0.0, 0.0, 0.0);
    let rate_calibration_number = 3000.0;

    for _ in 0..(rate_calibration_number as i32) {
        calibration_values.x += mpu.get_gyro().unwrap().x;
        calibration_values.y += mpu.get_gyro().unwrap().y;
        calibration_values.z += mpu.get_gyro().unwrap().z;
    }

    calibration_values.x /= 3000.0;
    calibration_values.y /= 3000.0;
    calibration_values.z /= 3000.0;

    calibration_values
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100_000),
        Default::default()
    );

    let mut delay = Delay;

    let mut mpu = Mpu6050::new(i2c, &mut delay);
    mpu.init().unwrap();

    let gyro_calibration_values = find_gyroscope_drift(&mut mpu);

    let mut last_time = Instant::now().as_micros() as f32 / 1_000_000.0;

    let mut roll_angle = 0.0;
    let mut pitch_angle = 0.0;
    let mut yaw_angle = 0.0;

    loop {
        let new_time = Instant::now().as_micros() as f32 / 1_000_000.0;
        let g = mpu.get_gyro().unwrap();

        let dt = new_time - last_time;
        last_time = new_time;

        let rate_roll = g.x - gyro_calibration_values.x;
        let rate_pitch = g.y - gyro_calibration_values.y;
        let rate_yaw = g.z - gyro_calibration_values.z;

        roll_angle += rate_roll * dt * 180.0 / PI;
        pitch_angle += rate_pitch * dt * 180.0 / PI;
        yaw_angle += rate_yaw * dt * 180.0 / PI;

        println!("{} {} {}", roll_angle, pitch_angle, yaw_angle);
    }
}