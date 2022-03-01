use std::io::{Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::new(9));
    let _ = e.write_all(data);
    e.finish().unwrap()
}

pub fn decompress(data: &[u8]) -> Vec<u8> {
    let mut d = ZlibDecoder::new(data);
    let mut buf = Vec::new();
    let _ = d.read_to_end(&mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known() {
        let original = vec![
            0x0c, 0x43, 0x36, 0x0c, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15,
            0x16, 0x15, 0x16,
        ];
        let compressed = vec![
            0x78, 0xda, 0xe3, 0x71, 0x36, 0xe3, 0xe1, 0x64, 0x80, 0x02, 0x51, 0x31, 0x51, 0x31,
            0x00, 0x0a, 0x2a, 0x00, 0xf1,
        ];
        assert_eq!(compress(&original), compressed);
        assert_eq!(decompress(&compressed), original);
    }
}
