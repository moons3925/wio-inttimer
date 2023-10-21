#![no_std]
#![no_main]

mod timer;
mod gpio;

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController; // GenericClockController
use wio::hal::timer::TimerCounter; // TimerCounter
use wio::pac::{interrupt, Peripherals, TC3};
use wio::prelude::*;
use wio::{entry, Pins, Sets};
use cortex_m::peripheral::NVIC;
use wio::prelude::_embedded_hal_timer_CountDown;
use crate::gpio::Led;

pub struct Ctx {
    led: Led,
    tc3: TimerCounter<TC3>,
}

pub static mut CTX: Option<Ctx> = None;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    // クロックを初期化する
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    // UARTドライバオブジェクトを初期化する
    let mut sets: Sets = Pins::new(peripherals.PORT).split();
    let mut serial = sets.uart.init(
        &mut clocks,
        115200.hz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        &mut sets.port,
    );

    // 2MHzのクロックを取得する
    let gclk5 = clocks
        .get_gclk(wio::pac::gclk::pchctrl::GEN_A::GCLK5)
        .unwrap();
    // TC3へのクロックを2MHzにする
    let timer_clock = clocks.tc2_tc3(&gclk5).unwrap();
    // TC3ドライバオブジェクトを初期化する
    let mut tc3 = TimerCounter::tc3_(
        &timer_clock,
        peripherals.TC3,
        &mut peripherals.MCLK,
    );

    // 割り込みコントローラで、TC3の割り込み通知を有効化する
    unsafe {
        NVIC::unmask(interrupt::TC3);
    }

    // 1秒のカウントを開始して、TC3の割り込みが発生するようにする
    tc3.start(1.s());
    tc3.enable_interrupt();

    // 割り込みハンドラと共有するリソースを格納する
    unsafe {
        CTX = Some(Ctx {
            led: Led::new(sets.user_led, &mut sets.port),
            tc3,
        });
    }

    // シリアルターミナルにechoし続ける
    loop {
        // データを1ワード受信するとifブロック内に入る
        if let Ok(c) = nb::block!(serial.read()) {
            // 受信したデータをそのまま送信する
            nb::block!(serial.write(c)).unwrap();
        }
    }
}

