use rand::Rng;
use std::fmt::Write;

pub enum Lettercase {
    Lowercase,
    Uppercase,
}

pub fn bytes_to_string(bytes: &Vec<u8>, letter_case: Lettercase, separator: &str) -> String {
    let mut result = String::new();

    for (i, byte) in bytes.iter().enumerate() {
        match letter_case {
            Lettercase::Lowercase => write!(&mut result, "{:02x}", byte),
            Lettercase::Uppercase => write!(&mut result, "{:02X}", byte),
        }
        .unwrap();

        if i != bytes.len() - 1 {
            result.push_str(separator);
        }
    }

    result
}

pub fn string_to_bytes(byte_string: &str) -> Vec<u8> {
    let mut bytes = vec![];
    let sanitized_string: String = byte_string.chars().filter(|c| !c.is_whitespace()).collect();

    if sanitized_string.is_empty() || sanitized_string.len() % 2 != 0 {
        return bytes;
    }

    bytes.reserve(sanitized_string.len() / 2);

    let mut rng = rand::thread_rng();
    let mut byte = String::with_capacity(2);

    for c in sanitized_string.chars() {
        if !c.is_digit(16) {
            write!(&mut byte, "{:X}", rng.gen_range(0..16)).unwrap();
        } else {
            byte.push(c);
        }

        if byte.len() == 2 {
            bytes.push(u8::from_str_radix(&byte, 16).unwrap());
            byte.clear();
        }
    }

    bytes
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn test_util_string_to_bytes() {
        assert_eq!(
            vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
            string_to_bytes(&"12 34 56 78 90 AB CD EF")
        );
    }

    #[test]
    fn test_util_bytes_to_string() {
        assert_eq!(
            "12 34 56 78 90 AB CD EF",
            bytes_to_string(
                &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
                Lettercase::Uppercase,
                " "
            )
        );
        assert_eq!(
            "1234567890abcdef",
            bytes_to_string(
                &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
                Lettercase::Lowercase,
                ""
            )
        );
        assert_eq!(
            "12*34*56*78*90*AB*CD*EF",
            bytes_to_string(
                &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
                Lettercase::Uppercase,
                "*"
            )
        );
        assert_eq!(
            "12--34--56--78--90--ab--cd--ef",
            bytes_to_string(
                &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
                Lettercase::Lowercase,
                "--"
            )
        );
    }
}
