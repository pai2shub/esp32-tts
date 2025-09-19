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
