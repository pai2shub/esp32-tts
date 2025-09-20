use std::sync::{Mutex, OnceLock};

// 录音/播放 采样率 HZ
pub const SAMPLE_RATE: u32 = 16000;

// paly gain
pub static PLAY_GAIN: OnceLock<Mutex<u8>> = OnceLock::new();

// lvgl

// LCD display
pub const DISPLAY_WIDTH: usize = 240;
pub const DISPLAY_HEIGHT: usize = 240;

// tts

//  TTS TEXT
pub const TTS_TEXT_HELLO: &str = "欢迎使用文字转转语音示例";

// server

// 嵌入index.html到二进制文件中
pub const INDEX_HTML: &str = include_str!("../assets/index.html");

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

pub fn init() {
    PLAY_GAIN.set(Mutex::new(1)).unwrap();
}
