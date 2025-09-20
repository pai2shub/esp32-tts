use esp_idf_svc::hal::{gpio::AnyIOPin, i2s::I2S1};

use std::sync::mpsc;
use std::thread::spawn;

mod audio;
mod button;
mod global;
mod server;
mod tts;
mod ui_lvgl;
mod utils;
mod wifi;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_svc::hal::prelude::Peripherals::take().unwrap();
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    utils::log_heap();

    log::info!("init esp32s3 tts demo");

    global::init();

    // init ui
    log::info!("init ui");
    let mut ui = ui_lvgl::UI::new();
    utils::log_heap();

    utils::print_partitions();

    // init button
    log::info!("init button");
    let mut btn_k0 = button::Button::new(peripherals.pins.gpio0.into(), button::ButtonType::K0)?;
    let mut btn_up = button::Button::new(peripherals.pins.gpio38.into(), button::ButtonType::Up)?;
    let mut btn_down =
        button::Button::new(peripherals.pins.gpio39.into(), button::ButtonType::Down)?;

    spawn(move || loop {
        log::info!("wait_for_any_edge btn_up");
        let e = btn_up.wait_for_any_edge();
        audio::volume_up();
        log::info!("wait_for_any_edge {:?}", e);
    });

    spawn(move || loop {
        log::info!("wait_for_any_edge btn_down");
        let e = btn_down.wait_for_any_edge();
        audio::volume_down();
        log::info!("wait_for_any_edge {:?}", e);
    });

    // tts text channel
    let (tx, rx) = mpsc::channel();
    // tts text to sound channel
    let (tx2, rx2) = mpsc::channel();
    // ui show text channel
    let (tx3, rx3) = mpsc::channel();

    // init audio
    log::info!("init audio");
    let i2s1: I2S1 = peripherals.i2s1;
    let dout: AnyIOPin = peripherals.pins.gpio7.into();
    let bclk: AnyIOPin = peripherals.pins.gpio15.into();
    let lrclk: AnyIOPin = peripherals.pins.gpio16.into();
    let mclk: Option<AnyIOPin> = None;

    let mut audio = audio::Audio::new(i2s1, dout, bclk, lrclk, mclk);
    spawn(move || {
        audio.play_with_tx(rx2);
    });
    utils::log_heap();

    // init tts
    log::info!("init tts");
    let mut tts = tts::TTS::new();
    spawn(move || {
        tts.play_with_rx(rx, tx2);
    });
    utils::log_heap();

    // init wifi ap
    log::info!("wifi ap");
    let wifi_ap = wifi::wifi_ap(peripherals.modem, sysloop.clone())?;
    log::info!("Wifi AP SSID: {:?}", global::WIFI_AP_NAME);
    log::info!("Wifi AP IP: {:?}", wifi_ap.ap_netif().get_ip_info()?);
    utils::log_heap();

    // show hello text
    _ = tx.clone().send(global::TTS_TEXT_HELLO.to_string());
    // speak hello
    _ = tx3.clone().send(global::TTS_TEXT_HELLO.to_string());

    // wait k0 button press
    log::info!("wait_for_any_edge btn_k0");
    let e = btn_k0.wait_for_any_edge();
    log::info!("wait_for_any_edge {:?}", e);

    // start server
    log::info!("start server");
    server::server(tx, tx3)?;
    utils::log_heap();

    // run ui
    log::info!("ui run");
    ui.run(rx3);

    log::error!("restart");
    unsafe { esp_idf_svc::sys::esp_restart() }
}
