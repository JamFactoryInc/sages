
pub const fn parse_u16(str: &'static str) -> u16 {
    if !str.is_ascii() {
        panic!("expected ascii string")
    }
    let bytes = str.as_bytes();
    let mut val = 0u16;
    let mut i = 0;
    while i < bytes.len() {
        val *= 10;
        val += (bytes[i] - b'0') as u16;
        i += 1;
    }
    val
}