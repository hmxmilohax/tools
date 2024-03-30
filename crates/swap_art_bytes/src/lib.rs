pub fn swap_art_bytes(buf: &mut [u8]) {
    for i in (32..buf.len()).step_by(2) {
        let byte1 = buf[i];
        let byte2 = buf[i + 1];

        buf[i] = byte2;
        buf[i + 1] = byte1;
    }
}
