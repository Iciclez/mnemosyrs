pub struct PatternMatch {
    pattern: String,
    pattern_size: usize,

    memory_start: *const u8,
    memory_size: usize,
    current_address: *mut u8,

    byte_array: Vec<u8>,
    mask: Vec<u8>,
}

impl PatternMatch {
    pub fn new(pattern: String, memory_start: *const u8, memory_size: usize) -> Self {
        let mut pattern_match = PatternMatch {
            pattern: pattern.clone(),
            pattern_size: 0,

            memory_start: memory_start,
            memory_size: memory_size,
            current_address: memory_start as *mut u8,

            byte_array: vec![],
            mask: vec![],
        };

        pattern_match.pattern = pattern_match
            .pattern
            .trim_end_matches(|c: char| c == '?' || c.is_whitespace())
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        if pattern_match.pattern.is_empty() || pattern_match.pattern.len() % 2 == 1 {
            panic!("pattern is in unexpected format");
        }

        pattern_match.pattern_size = pattern_match.pattern.len() / 2;

        pattern_match.byte_array.reserve(pattern_match.pattern_size);
        pattern_match.mask.reserve(pattern_match.pattern_size);

        for n in (0..pattern_match.pattern.len()).step_by(2) {
            if pattern_match.pattern.chars().nth(n).unwrap() == '?'
                || pattern_match.pattern.chars().nth(n + 1).unwrap() == '?'
            {
                pattern_match.mask.push(1);
                pattern_match.byte_array.push(0);
            } else {
                pattern_match.mask.push(0);
                pattern_match.byte_array.push(
                    u8::from_str_radix(&pattern_match.pattern[n..n + 2], 16)
                        .ok()
                        .unwrap(),
                );
            }
        }

        pattern_match
    }

    pub fn find_address(&mut self) -> *const u8 {
        unsafe { self.find_address_from(self.memory_start as *mut u8) }
    }

    pub fn find_next_address(&mut self) -> *const u8 {
        unsafe { self.find_address_from(self.current_address.add(1)) }
    }

    unsafe fn find_address_from(&mut self, address_from: *mut u8) -> *const u8 {
        for offset in 0..self.memory_size {
            self.current_address = address_from.add(offset);

            if self.try_match_at_current_address() {
                return self.current_address;
            }
        }

        std::ptr::null()
    }

    unsafe fn try_match_at_current_address(&mut self) -> bool {
        let mut j = 0;

        while j < self.pattern_size
            && (self.mask[j] == 0x01 || &*self.current_address.add(j) ^ self.byte_array[j] == 0)
        {
            j += 1;
        }

        j == self.pattern_size
    }
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn test_pattern_match_new() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let pattern_match = PatternMatch::new(
            String::from("0a 0b ?? ??   ?? 2e ??   ?? "),
            std::ptr::null(),
            0 as usize,
        );
        let sanitized_pattern = "0a0b??????2e";
        assert_eq!(pattern_match.pattern, sanitized_pattern);
        assert_eq!(pattern_match.pattern_size, sanitized_pattern.len() / 2);
    }

    #[test]
    fn test_pattern_match_find_address() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let mut haystack = vec![
            0xf1, 0x80, 0xd7, 0x50, 0x1a, 0x7b, 0x69, 0x57, 0x07, 0x80, 0xbc, 0x27, 0xc7, 0x5e,
            0x88, 0x0c, 0xac, 0x7f, 0xd8, 0xe0, 0x13, 0x7d, 0xf4, 0xfb, 0xf4, 0x91, 0x0b, 0x07,
            0xa6, 0xe1, 0x54, 0x22,
        ];

        unsafe {
            assert_eq!(
                haystack.as_mut_ptr().add(5) as *const u8,
                PatternMatch::new(
                    String::from("7b ?? 57 07 ?? bc ?? c7"),
                    haystack.as_ptr(),
                    haystack.len()
                )
                .find_address()
            );
            assert_eq!(
                std::ptr::null(),
                PatternMatch::new(
                    String::from("7b ?? 57 33 07 ?? bc ?? c7"),
                    haystack.as_ptr(),
                    haystack.len()
                )
                .find_address()
            );
        }
    }

    #[test]
    fn test_pattern_match_find_next_address() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let mut haystack = vec![
            0xf1, 0x80, 0xd7, 0x50, 0x1a, 0x7b, 0x69, 0x57, 0x07, 0x80, 0xbc, 0x27, 0xc7, 0x5e,
            0x88, 0x0c, 0xac, 0x7f, 0xd8, 0xe0, 0x13, 0x7d, 0xf4, 0xfb, 0xf4, 0x91, 0x0b, 0x07,
            0xa6, 0xe1, 0x54, 0x22, 0x7b, 0xa0, 0x57, 0x07, 0x2b, 0xbc, 0xdd, 0xc7, 0x24, 0x53,
            0xb3, 0x3f, 0xf1, 0xd5, 0x67, 0x23,
        ];

        let mut pattern_match = PatternMatch::new(
            String::from("7b ?? 57 07 ?? bc ?? c7"),
            haystack.as_ptr(),
            haystack.len(),
        );

        // next result
        unsafe {
            assert_eq!(
                haystack.as_mut_ptr().add(5) as *const u8,
                pattern_match.find_address()
            );
            assert_eq!(
                haystack.as_mut_ptr().add(32) as *const u8,
                pattern_match.find_next_address()
            );
        }
    }
}
