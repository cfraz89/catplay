pub fn pretty_hexdump(data: &[u8]) -> String {
    const BYTES_PER_LINE: usize = 16;

    let mut out = String::with_capacity(data.len() * 4); // estimate

    let hex = b"0123456789abcdef";

    for (i, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        // offset
        let offset = i * BYTES_PER_LINE;
        push_hex_u32(&mut out, offset as u32);
        out.push_str(": ");

        // hex bytes
        for &b in chunk {
            out.push(hex[(b >> 4) as usize] as char);
            out.push(hex[(b & 0xF) as usize] as char);
            out.push(' ');
        }

        // padding
        for _ in chunk.len()..BYTES_PER_LINE {
            out.push_str("   ");
        }

        // ascii
        out.push('|');
        for &b in chunk {
            if b.is_ascii_graphic() || b == b' ' {
                out.push(b as char);
            } else {
                out.push('.');
            }
        }
        out.push('|');
        out.push('\n');
    }

    out
}

pub fn pretty_hexdump_limited(data: &[u8], max: usize) -> String {
    if data.len() > max {
        let mut s = pretty_hexdump(&data[..max]);
        s += &format!(" ... was a partial trace of total {} bytes ...\n", data.len());
        s
    } else {
        pretty_hexdump(data)
    }
}

fn push_hex_u32(out: &mut String, mut v: u32) {
    let hex = b"0123456789abcdef";
    let mut buf = [0u8; 8];

    for i in (0..8).rev() {
        buf[i] = hex[(v & 0xF) as usize];
        v >>= 4;
    }

    for b in &buf {
        out.push(*b as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_hexdump_basic() {
        let data = b"Hello, world!\n";

        let dump = pretty_hexdump(data);

        let expected = "00000000: 48 65 6c 6c 6f 2c 20 77 6f 72 6c 64 21 0a       |Hello, world!.|\n";

        assert_eq!(dump, expected);
    }

    #[test]
    fn test_pretty_hexdump_two_lines() {
        let data: Vec<u8> = (0u8..32u8).collect();

        let dump = pretty_hexdump(&data);

        let expected = concat!(
            "00000000: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f |................|\n",
            "00000010: 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f |................|\n"
        );

        assert_eq!(dump, expected);
    }

    #[test]
    fn test_pretty_hexdump_padding() {
        let data = &[0xde, 0xad, 0xbe];

        let dump = pretty_hexdump(data);

        let expected = "00000000: de ad be                                        |...|\n";

        assert_eq!(dump, expected);
    }

    #[test]
    fn test_pretty_hexdump_limited() {
        let data = &[0xde, 0xad, 0xbe];

        let dump = pretty_hexdump_limited(data, 2);

        let expected = "00000000: de ad                                           |..|\n ... was a partial trace of total 3 bytes ...\n";

        assert_eq!(dump, expected);
    }
}
