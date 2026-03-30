pub trait CanonicalEncode {
    fn encode(&self, out: &mut Vec<u8>);

    fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.encode(&mut out);
        out
    }
}

pub fn canonical_bytes<T: CanonicalEncode>(value: &T) -> Vec<u8> {
    value.to_canonical_bytes()
}

pub fn write_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

pub fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn write_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn write_fixed<const N: usize>(out: &mut Vec<u8>, value: &[u8; N]) {
    out.extend_from_slice(value);
}

pub fn write_bytes(out: &mut Vec<u8>, value: &[u8]) {
    write_u32(out, value.len() as u32);
    out.extend_from_slice(value);
}

pub fn write_option_bytes(out: &mut Vec<u8>, value: Option<&[u8]>) {
    match value {
        Some(value) => {
            write_u8(out, 1);
            write_bytes(out, value);
        }
        None => write_u8(out, 0),
    }
}

pub fn write_vec<T: CanonicalEncode>(out: &mut Vec<u8>, values: &[T]) {
    write_u32(out, values.len() as u32);
    for value in values {
        value.encode(out);
    }
}

pub fn write_vec_bytes(out: &mut Vec<u8>, values: &[Vec<u8>]) {
    write_u32(out, values.len() as u32);
    for value in values {
        write_bytes(out, value);
    }
}
