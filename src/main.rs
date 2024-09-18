#![no_std]
#![no_main]

mod button;

use button::ButtonDirection;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use panic_rtt_target as _;

static CHANNEL: Channel<ThreadModeRawMutex, ButtonDirection, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    spawner
        .spawn(button_task(p.P0_14.degrade(), ButtonDirection::Left))
        .unwrap();
    spawner
        .spawn(button_task(p.P0_23.degrade(), ButtonDirection::Right))
        .unwrap();

    // Toggle LED
    let mut led = led_pin(p.P0_28.degrade());

    loop{
        led.toggle();
        _ = Timer::after_millis(500);
    }
}

fn led_pin(pin: AnyPin) -> Output<'static> {
    Output::new(pin, Level::High, OutputDrive::Standard)
}


#[embassy_executor::task(pool_size = 2)]
async fn button_task(
    pin: AnyPin,
    direction: ButtonDirection,
) {
    let mut input = Input::new(pin, Pull::None);
    loop {
        input.wait_for_low().await;
        CHANNEL.send(direction).await;
        Timer::after_millis(100).await;
        input.wait_for_high().await;
    }
}