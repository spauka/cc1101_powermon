const SCALE_KW: f32 = 0.4799;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecodeResult {
    pub power_kw: f32,
    pub packet: [u8; 8],
    pub quality_metric: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    NotEnoughData,
    PreambleNotFound,
    SyncNotFound,
    InsufficientSymbols,
    ChecksumMismatch { expected: u8, actual: u8 },
}

pub fn decode_power(rx_buf: &[u8; 128], read_bytes: usize) -> Result<DecodeResult, DecodeError> {
    if read_bytes == 0 || read_bytes > rx_buf.len() {
        return Err(DecodeError::NotEnoughData);
    }
    let bit_len = read_bytes * 8;
    let buf = &rx_buf[..read_bytes];

    let (preamble_end, quality) =
        find_preamble(buf, bit_len).ok_or(DecodeError::PreambleNotFound)?;

    let sync_end = find_sync(buf, preamble_end, bit_len).ok_or(DecodeError::SyncNotFound)?;

    let (packet, _) =
        demodulate_symbols(buf, sync_end, bit_len).ok_or(DecodeError::InsufficientSymbols)?;

    let expected_checksum = sum_checksum(&packet);
    if expected_checksum != packet[7] {
        return Err(DecodeError::ChecksumMismatch {
            expected: expected_checksum,
            actual: packet[7],
        });
    }

    let power = decode_word_from_packet(&packet);
    Ok(DecodeResult {
        power_kw: power,
        packet,
        quality_metric: quality,
    })
}

fn find_preamble(buf: &[u8], bit_len: usize) -> Option<(usize, u8)> {
    let mut start = 0usize;
    while start + 32 <= bit_len {
        let mut idx = start;
        let mut quality: u8 = 0;
        while idx + 4 <= bit_len && matches_1100(buf, idx) {
            quality = quality.saturating_add(1);
            idx += 4;
        }
        if quality >= 8 {
            return Some((idx, quality));
        }
        start += 1;
    }
    None
}

fn matches_1100(buf: &[u8], bit_idx: usize) -> bool {
    bit_at(buf, bit_idx) == 1
        && bit_at(buf, bit_idx + 1) == 1
        && bit_at(buf, bit_idx + 2) == 0
        && bit_at(buf, bit_idx + 3) == 0
}

fn find_sync(buf: &[u8], start_bit: usize, bit_len: usize) -> Option<usize> {
    if bit_len < 16 || start_bit >= bit_len - 15 {
        return None;
    }

    let mut window: u32 = 0;
    for idx in start_bit..bit_len {
        window = ((window << 1) | bit_at(buf, idx) as u32) & 0xFFFF;
        if idx + 1 >= start_bit + 16 && window == 0xFFFF {
            return Some(idx + 1);
        }
    }
    None
}

fn demodulate_symbols(
    buf: &[u8],
    mut index: usize,
    bit_len: usize,
) -> Option<([u8; 8], usize)> {
    let mut packet = [0u8; 8];
    let mut bit_count = 0usize;

    while index < bit_len && bit_count < 64 {
        let mut zeros = 0usize;
        while index < bit_len && bit_at(buf, index) == 0 {
            zeros += 1;
            index += 1;
        }

        let mut ones = 0usize;
        while index < bit_len && bit_at(buf, index) == 1 {
            ones += 1;
            index += 1;
        }

        if zeros == 0 || ones == 0 {
            break;
        }

        let bit = if zeros >= ones { 0 } else { 1 };
        let byte_idx = bit_count / 8;
        packet[byte_idx] = (packet[byte_idx] << 1) | bit;
        bit_count += 1;
    }

    if bit_count < 64 {
        return None;
    }
    Some((packet, index))
}

fn sum_checksum(packet: &[u8; 8]) -> u8 {
    let mut acc = 0u8;
    for &b in &packet[..7] {
        acc = acc.wrapping_add(b);
    }
    acc
}

fn decode_word_from_packet(packet: &[u8; 8]) -> f32 {
    let mantissa = u16::from_be_bytes([packet[4], packet[5]]);
    let exponent = packet[6] as i32 - 1;
    let mantissa_ratio = mantissa as f32 / 65536.0;
    SCALE_KW * mantissa_ratio * ((2 << exponent) as f32)
}

#[inline(always)]
fn bit_at(buf: &[u8], bit_idx: usize) -> u8 {
    let byte_idx = bit_idx / 8;
    let bit_in_byte = 7 - (bit_idx % 8);
    (buf[byte_idx] >> bit_in_byte) & 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_packets() {
        let vectors: &[(&str, f32)] = &[
            (
                "33 33 33 33 33 33 33 33 30 00 00 00 00 03 00 00 00 60 00 00 00 00 00 00 00 7F FF 83 0C 18 67 83 0C F1 E0 C3 3C F8 67 DF 06 19 F0 CF 3E F8 60 CF 86 18 30 C3 06 0C F3 CF 9E 79 F0 C7 86 7C F3 C1 86 08 30 61 83 0C 30 67 83 3C 18 67 C3 0C F9 3C F3 FB C4",
                0.475096,
            ),
            (
                "33 33 33 33 33 33 33 30 00 00 00 00 03 00 00 00 60 00 00 00 00 00 00 00 7F FF 83 0C 18 67 83 0C F1 E0 C3 3C F8 67 DF 06 19 F0 CF 3E 78 60 CF 86 18 30 C3 06 0C F3 CF 9E 0D F3 C7 83 0C 30 67 87 7C 18 61 C3 0C 18 67 83 06 19 E0 C3 3C F9 7B 2F D5",
                0.463102,
            ),
            (
                "66 66 66 66 66 66 66 66 00 00 00 00 00 60 00 00 0C 00 00 00 00 00 00 00 0F FF F0 61 83 0C F8 61 9F 3C 18 67 CF 0C F9 E0 C3 3E 19 E7 CF 0C 19 F0 C3 06 18 60 C1 9E 79 F3 CF 06 78 33 CF 9E 18 30 C1 06 0C 30 61 86 0C F0 61 83 0C 30 61 82 18 93 D5 D0 3D 13 0A 6A AA D6 D1 05 FA 6C CE B0 C6 90",
                0.470293,
            ),
            (
                "66 66 66 66 66 66 66 66 00 00 00 00 00 60 00 00 0C 00 00 00 00 00 00 00 0F FF F0 61 83 0C F0 61 9E 3C 18 67 9F 0C FB E0 C3 3E 19 E7 CF 0C 19 F0 C3 06 18 60 C1 9E 0C 30 C1 86 18 30 61 87 0C 30 61 83 0C 18 61 83 3C 10 67 C3 3C 19 E0 C3 2F FC FE 9B AF 5D 38 DD C1 89 C8 D1 87 B2 B9 D3 D0 62 2B 0C ED BF C2 85 DA",
                0.479900,
            ),
            (
                "CC CC CC CC CC CC CC 00 00 00 00 00 C0 00 00 18 00 00 00 00 00 00 00 1F FF E0 C3 06 19 E0 C3 3C 78 30 CF BE 19 F3 C1 86 7C 33 CF 9E 18 33 E1 86 0C 30 C1 83 3C 10 61 83 0C 30 60 C3 0C 18 61 C3 06 18 30 C3 06 78 60 CF 86 78 33 C3 86 39 0E DF E5 70 D9 27 E4 A6",
                0.479900,
            ),
            (
                "66 66 66 66 66 66 66 60 00 00 00 00 06 00 00 00 C0 00 00 00 00 00 00 00 FF FF 06 18 30 CF 86 19 F3 E1 86 7C F0 CF 9E 0C 33 E1 9E 7C F0 C1 9F 0C 30 61 83 0C 19 E0 C3 0C 18 67 8F 3E 78 67 C3 3C F8 30 C3 06 18 37 C3 06 78 30 C1 86 18 33 97 F9 D7 5D FD D6 87 A7 51 5A 29 08 A2 CB 74 8F",
                0.494282,
            ),
        ];

        for (hex_str, expected_power) in vectors {
            let bytes = parse_hex(hex_str);
            let mut buf = [0u8; 128];
            buf[..bytes.len()].copy_from_slice(&bytes);
            let result = decode_power(&buf, bytes.len()).expect("decoder should succeed");
            println!("Exp: {}, Act: {}", expected_power, result.power_kw);
            assert!((result.power_kw - expected_power).abs() < 1e-4);
            assert!(result.quality_metric >= 8);
        }
    }

    #[test]
    fn preamble_in_middle() {
        let hex_str = "10 5F 94 78 F9 C7 98 59 5E 3C 94 43 1A 96 BE 3E FC CC CC CC CC CC CC CC CC 00 00 00 00 00 C0 00 00 18 00 00 00 00 00 00 00 1F FF E0 C3 06 19 F0 C3 3E 7C 30 CF 9E 19 F3 C1 86 7C 33 CF 9E 18 33 E1 86 0C 30 C1 83 3C F0 67 9F 7C 30 61 C3 3C 19 E1 83 06 18 70 C3 06 19 E3 C1 86 78 33 C1 9E 39 3D 34 A7 DE D1 1A 2A BD";
        let expected_power: f32 = 0.4127;
        let bytes = parse_hex(hex_str);
        let mut buf = [0u8; 128];
        buf[..bytes.len()].copy_from_slice(&bytes);
        let result = decode_power(&buf, bytes.len()).expect("decoder should succeed");
        println!("Exp: {}, Act: {}", expected_power, result.power_kw);
        assert!((result.power_kw - expected_power).abs() < 1e-4);
        assert!(result.quality_metric >= 8);
    }

    #[test]
    fn rejects_too_short() {
        let hex_str = "99 99 99 99 99 99 99 98 00 00 00 00 01 80 00 00 30 00 00 00 00 00 00 00 3F FF C1 86 0C 33 E1 86 78 F0 61 9F 7C 33 E7 83 0C";
        let bytes = parse_hex(hex_str);
        let mut buf = [0u8; 128];
        buf[..bytes.len()].copy_from_slice(&bytes);
        let err = decode_power(&buf, bytes.len()).expect_err("decoder should fail");
        assert!(matches!(err, DecodeError::InsufficientSymbols { .. }));
    }

    #[test]
    fn rejects_bad_checksum() {
        let hex_str = "33 33 33 33 33 33 33 30 00 00 00 00 03 00 00 00 60 00 00 00 00 00 00 00 7F FF 83 0C 18 67 C3 0C F1 E0 C3 3E F8 67 DF 06 19 F0 FC 3E 78 60 CF 86 18 30 C3 06 0C F3 C1 9E 7D F3 C1 9E 0C F3 E7 86 0C 18 61 83 0C 18 67 8F 3C 19 E7 CF 3C 18 5D";
        let bytes = parse_hex(hex_str);
        let mut buf = [0u8; 128];
        buf[..bytes.len()].copy_from_slice(&bytes);
        let err = decode_power(&buf, bytes.len()).expect_err("decoder should fail");
        assert!(matches!(err, DecodeError::ChecksumMismatch { .. }));
    }

    fn parse_hex(input: &str) -> Vec<u8> {
        input
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16).expect("valid hex"))
            .collect()
    }
}
