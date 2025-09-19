use esp_idf_svc::hal::i2s::I2sDriver;

use std::sync::mpsc;

use crate::global;

pub struct Audio {
    tx_driver: I2sDriver<'_, I2sTx>,
}

impl Audio {
    pub fn new(
        i2s1: I2S1,
        dout: AnyIOPin,
        bclk: AnyIOPin,
        lrclk: AnyIOPin,
        mclk: Option<AnyIOPin>,
    ) -> Self {
        let i2s_config = config::StdConfig::new(
            config::Config::default().auto_clear(true),
            config::StdClkConfig::from_sample_rate_hz(SAMPLE_RATE),
            config::StdSlotConfig::philips_slot_default(
                config::DataBitWidth::Bits16,
                config::SlotMode::Mono,
            ),
            config::StdGpioConfig::default(),
        );

        let mut tx_driver =
            I2sDriver::new_std_tx(i2s1, &i2s_config, bclk, dout, mclk, lrclk).unwrap();
        log::info!("I2S driver initialized");

        tx_driver.tx_enable().unwrap();
        log::info!("I2S driver enabled");

        Audio { tx_driver }
    }

    fn play(self, data: &[u8]) {
        amplify_pcm_data(&mut data, global::PLAY_GAIN);
        tx_driver.write_all(data, 1000).unwrap();
    }

    pub fn play_with_tx(self, tx: mpsc::Receiver<&[u8]>) {
        loop {
            let data = tx.recv().unwrap();
            self.play(data);
        }
    }
}

// 放大声音
fn amplify_pcm_data(input: &mut [u8], gain: u8) {
    // let gain = 1;
    // 每次处理 2 个字节（1 个 i16 样本）的可变切片
    for chunk in input.chunks_exact_mut(2) {
        // 将 u8 转换为 i16（小端模式）
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        // 应用增益
        let amplified = sample as i32 * gain as i32;
        // 钳位处理（防止溢出 i16 范围）
        let clamped = amplified.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        // 将处理后的样本写回原数组
        let bytes = clamped.to_le_bytes();
        chunk[0] = bytes[0];
        chunk[1] = bytes[1];
    }
}
