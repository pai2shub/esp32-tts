use esp_idf_svc::{
    hal::{
        gpio::{AnyIOPin, Gpio0, Input, PinDriver},
        i2s::{config, I2sDriver, I2S0, I2S1},
    },
    io::Read,
    sys::esp_sr,
};
use std::ffi::{CStr, CString};
use std::{ptr, slice};

mod ui_lvgl;

const SAMPLE_RATE: u32 = 16000;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_svc::hal::prelude::Peripherals::take().unwrap();

    log::info!("Hello, world!");

    let mut ui = ui_lvgl::UI::new();
    log_heap();

    unsafe {
        log::info!("esp_partition_find_first");
        // 开始查找：传 null 表示从头开始
        let mut iterator = esp_sr::esp_partition_find(
            esp_sr::esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
            esp_sr::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
            ptr::null(),
        );
        if iterator.is_null() {
            log::error!("Couldn't find any partitions!");
            return;
        }

        while !iterator.is_null() {
            let part = esp_sr::esp_partition_get(iterator);
            if !part.is_null() {
                // 获取分区信息
                let partition = *part; // 解引用 C struct
                let name_cstr = CStr::from_ptr(partition.label.as_ptr());
                let name = name_cstr.to_string_lossy();

                log::info!(
                    "Partition: name={}, type=0x{:X}, subtype=0x{:X}, offset=0x{:X}, size={} bytes",
                    name,
                    partition.type_ as u8,
                    partition.subtype as u8,
                    partition.address,
                    partition.size
                );
            }

            // 继续查找下一个
            iterator = esp_sr::esp_partition_next(iterator);
        }
        esp_sr::esp_partition_iterator_release(iterator);
    }

    let i2s_config = config::StdConfig::new(
        config::Config::default().auto_clear(true),
        config::StdClkConfig::from_sample_rate_hz(SAMPLE_RATE),
        config::StdSlotConfig::philips_slot_default(
            config::DataBitWidth::Bits16,
            config::SlotMode::Mono,
        ),
        config::StdGpioConfig::default(),
    );

    let i2s1: I2S1 = peripherals.i2s1;
    let dout: AnyIOPin = peripherals.pins.gpio7.into();
    let bclk: AnyIOPin = peripherals.pins.gpio15.into();
    let lrclk: AnyIOPin = peripherals.pins.gpio16.into();
    let mclk: Option<AnyIOPin> = None;

    let mut tx_driver = I2sDriver::new_std_tx(i2s1, &i2s_config, bclk, dout, mclk, lrclk).unwrap();
    log::info!("I2S driver initialized");

    tx_driver.tx_enable().unwrap();
    log::info!("I2S driver enabled");

    unsafe {
        log::info!("esp_tts_init");

        let partition_name = CString::new("voice_data").unwrap();
        let pt = esp_sr::esp_partition_find_first(
            esp_sr::esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
            esp_sr::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
            partition_name.as_ptr(),
        );
        if pt.is_null() {
            log::error!(
                "Couldn't find voice data partition! {}",
                partition_name.to_str().unwrap()
            );
            return;
        }
        log::info!(
            "esp partition find first {}",
            partition_name.to_str().unwrap()
        );

        let mut voicedata: *const std::ffi::c_void = std::ptr::null();
        let mut mmap_handle: esp_sr::esp_partition_mmap_handle_t = std::mem::zeroed();

        let err = esp_sr::esp_partition_mmap(
            pt,
            0,
            (*pt).size as usize,
            esp_sr::esp_partition_mmap_memory_t_ESP_PARTITION_MMAP_DATA,
            &mut voicedata as *mut *const std::ffi::c_void,
            &mut mmap_handle as *mut _,
        );
        if err != esp_sr::ESP_OK {
            log::error!("Couldn't map voice data partition!");
            return;
        }
        log::info!("esp partition mmap initialized");

        let voicedata_mut = voicedata as *mut std::ffi::c_void;
        let voice = esp_sr::esp_tts_voice_set_init(
            &esp_sr::esp_tts_voice_template as *const _,
            voicedata_mut,
        );
        log::info!("esp_tts_voice_set_init");

        let tts_handle = esp_sr::esp_tts_create(voice);
        log::info!("esp_tts_create");

        let prompt =
            CString::new("嵌入式螃蟹训练营第二期专用设备的租用价格是一百六十八元").unwrap();
        log::info!("prompt: {}", prompt.to_str().unwrap());

        if esp_sr::esp_tts_parse_chinese(tts_handle, prompt.as_ptr()) != 0 {
            let mut len = [0i32; 1];
            loop {
                let pcm_data = esp_sr::esp_tts_stream_play(tts_handle, len.as_mut_ptr(), 3);
                if len[0] <= 0 {
                    break;
                }

                // play sound
                let pcm_slice: &[u8] = slice::from_raw_parts(
                    pcm_data as *const u8, // 转为字节指针
                    (len[0] * 2) as usize, // 总字节数
                );
                tx_driver.write_all(&pcm_slice, 1000).unwrap();
            }
        }

        esp_sr::esp_tts_stream_reset(tts_handle);
        esp_sr::esp_partition_munmap(mmap_handle);
    }
}
