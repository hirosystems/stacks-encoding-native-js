pub fn decode_hex<T: AsRef<[u8]>>(data: T) -> Result<Box<[u8]>, hex_simd::Error> {
    let data_ref = data.as_ref();
    let data_len = data_ref.len();
    if data_len == 0 {
        Ok(Box::new([0u8; 0]))
    } else if data_len >= 2 && data_ref[0] == '0' as u8 && data_ref[1] == 'x' as u8 {
        hex_simd::decode_to_boxed_bytes(&data_ref[2..])
    } else {
        hex_simd::decode_to_boxed_bytes(data_ref)
    }
}

pub fn encode_hex(data: &[u8]) -> Box<str> {
    let mut uninit_buf = unsafe { simd_abstraction::tools::alloc_uninit_bytes(data.len() * 2 + 2) };
    let uninit_slice = &mut *uninit_buf;
    uninit_slice[0].write(b'0');
    uninit_slice[1].write(b'x');
    let dest_buf = hex_simd::OutBuf::from_uninit_mut(&mut uninit_slice[2..]);
    hex_simd::encode(data, dest_buf, hex_simd::AsciiCase::Lower).unwrap();

    let len = uninit_buf.len();
    let ptr = Box::into_raw(uninit_buf).cast::<u8>();
    unsafe {
        let buf = core::slice::from_raw_parts_mut(ptr, len);
        Box::from_raw(core::str::from_utf8_unchecked_mut(buf))
    }
}

pub fn encode_hex_no_prefix(data: &[u8]) -> Box<str> {
    hex_simd::encode_to_boxed_str(data, hex_simd::AsciiCase::Lower)
}
