use std::sync::mpsc;

pub struct TTS {
    voicedata: *const std::ffi::c_void,
    mmap_handle: esp_sr::esp_partition_mmap_handle_t,
}

impl TTS {
    pub fn new() -> Self {
        let mut voicedata: *const std::ffi::c_void = std::ptr::null();
        let mut mmap_handle: esp_sr::esp_partition_mmap_handle_t;

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
        }

        TTS {
            voicedata,
            mmap_handle,
        }
    }

    pub fn play(self, data: String, tx: mpsc::Sender<&[u8]>) {
        unsafe {
            let prompt = CString::new(data.as_str()).unwrap();
            log::info!("prompt: {}", prompt.to_str().unwrap());

            if esp_sr::esp_tts_parse_chinese(tts_handle, prompt.as_ptr()) == 0 {
                log::error!("esp_tts_parse_chinese fail");
            }

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

                tx.send(pcm_slice);
            }
        }
    }

    pub fn play_with_rx(self, rx: mpsc::Receiver<String>, tx: mpsc::Sender<&[u8]>) {
        loop {
            let data = rx.recv().unwrap();
            self.play(data, tx);
        }
    }

    pub fn drop(self) {
        let mut voicedata = self.voicedata;
        let mut mmap_handle = self.mmap_handle;

        unsafe {
            esp_sr::esp_tts_stream_reset(tts_handle);
            esp_sr::esp_partition_munmap(mmap_handle);
        }
    }
}
