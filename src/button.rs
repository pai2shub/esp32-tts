use core::{
    sync::atomic::{AtomicBool, Ordering},
    time,
};
use std::thread::sleep;

use anyhow::Ok;

use esp_idf_svc::hal::gpio::{AnyIOPin, Input, Pin, PinDriver};
use esp_idf_sys::EspError;

#[derive(Debug, Clone, Copy)]
pub enum ButtonType {
    K0,
    Up,
    Down,
}

#[derive(Debug)]
pub enum ButtonEvent {
    AnyEdge(ButtonType),
}

pub struct Button {
    typ: ButtonType,
    btn: PinDriver<'static, AnyIOPin, Input>,
    flag: &'static AtomicBool,
}

impl Button {
    pub fn new(gpio: AnyIOPin, typ: ButtonType) -> anyhow::Result<Button> {
        log::info!("new Button: {:?} {:?}", gpio.pin(), typ);
        // Configures the button
        let mut btn = esp_idf_svc::hal::gpio::PinDriver::input(gpio)?;
        btn.set_pull(esp_idf_svc::hal::gpio::Pull::Up)?;
        btn.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::PosEdge)?;

        let flag: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));

        let mut btn = Button { btn, typ, flag };
        btn.subscribe()?;

        Ok(btn)
    }

    // 获取按键信号
    pub fn wait_for_any_edge(&mut self) -> ButtonEvent {
        loop {
            if self.flag.swap(false, Ordering::Relaxed) {
                break;
            }
            sleep(time::Duration::from_millis(10));
        }

        // 一次中断触发后，会自动禁用中断，需要重新启用
        self.btn.enable_interrupt().unwrap();

        let t = self.typ;
        ButtonEvent::AnyEdge(t)
    }

    // 订阅中断处理
    fn subscribe(&mut self) -> anyhow::Result<(), EspError> {
        log::info!("Button {:?} subscribe", self.typ);

        unsafe {
            // 订阅中断处理
            self.btn.subscribe(|| {
                self.flag.store(true, Ordering::Relaxed);
            })?;
        }
        // 启用中断
        self.btn.enable_interrupt()
    }
}