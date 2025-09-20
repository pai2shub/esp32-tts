use esp_idf_svc::{
    hal::{
        gpio::{AnyIOPin, Gpio0, Input, PinDriver},
        i2s::{config, I2sDriver, I2S0, I2S1},
    },
    io::Read,
    sys::esp_sr,
};
use std::{
    ffi::{CStr, CString},
    os::unix::thread,
    thread::spawn,
};
use std::{ptr, slice};

use std::sync::mpsc;

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

    log::info!("init esp32s3 tts demo");

    global::init();

    let mut ui = ui_lvgl::UI::new();
    utils::log_heap();

    utils::print_partitions();

    let mut ui = ui_lvgl::UI::new();

    let mut btn_k0 = button::Button::new(peripherals.pins.gpio0.into(), button::ButtonType::K0)?;
    let mut btn_up = button::Button::new(peripherals.pins.gpio38.into(), button::ButtonType::Up)?;
    let mut btn_down =
        button::Button::new(peripherals.pins.gpio39.into(), button::ButtonType::Down)?;

    thread::spawn(move || loop {
        log::info!("wait_for_any_edge btn_k0");
        let e = btn_k0.wait_for_any_edge();
        log::info!("wait_for_any_edge {:?}", e);
    });

    thread::spawn(move || loop {
        log::info!("wait_for_any_edge btn_up");
        let e = btn_up.wait_for_any_edge();
        audio::volume_up();
        log::info!("wait_for_any_edge {:?}", e);
    });

    thread::spawn(move || loop {
        log::info!("wait_for_any_edge btn_down");
        let e = btn_down.wait_for_any_edge();
        audio::volume_down();
        log::info!("wait_for_any_edge {:?}", e);
    });

    log::info!("init audio");
    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx3, rx3) = mpsc::channel();

    // init audio
    let i2s1: I2S1 = peripherals.i2s1;
    let dout: AnyIOPin = peripherals.pins.gpio7.into();
    let bclk: AnyIOPin = peripherals.pins.gpio15.into();
    let lrclk: AnyIOPin = peripherals.pins.gpio16.into();
    let mclk: Option<AnyIOPin> = None;

    let audio = audio::Audio::new(i2s1, dout, bclk, lrclk, mclk);
    spawn(|| {
        audio.play_with_tx(rx2);
    });

    let tts = tts::TTS::new();
    spawn(|| {
        tts.play_with_rx(rx, tx2);
    });

    let wifi_ap = wifi::wifi_ap(peripherals.modem, sysloop.clone())?;

    log::info!("Wifi AP SSID: {:?}", constant::WIFI_AP_NAME);
    log::info!("Wifi AP IP: {:?}", wifi_ap.ap_netif().get_ip_info()?);

    utils::log_heap();

    tx.clone().send(global::TTS_TEXT_HELLO.to_string());
    tx3.clone().send(global::TTS_TEXT_HELLO.to_string());

    server::server(tx, tx3)?;
    utils::log_heap();

    ui.run(rx3);

    log::error!("restart");
    unsafe { esp_idf_svc::sys::esp_restart() }
}
