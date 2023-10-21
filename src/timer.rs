use wio_terminal as wio;
use wio::pac::interrupt;
use wio::prelude::_embedded_hal_timer_CountDown;

// TC3の割り込みハンドラ
// 1秒ごとに呼ばれる
#[interrupt]
fn TC3() {
    unsafe {
        let ctx = crate::CTX.as_mut().unwrap();
        // 次のカウントを開始する
        ctx.tc3.wait().unwrap();
        ctx.led.toggle();
    }
}
