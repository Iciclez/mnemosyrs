pub struct Address {
    ptr: *mut u8,
}

impl Address {
    pub fn new(ptr: *mut u8) -> Self {
        Address { ptr: ptr }
    }

    pub unsafe fn read_memory(&mut self, size: usize) -> Vec<u8> {
        let mut memory = vec![];
        memory.reserve(size);

        for i in 0..size {
            memory.push(*self.ptr.add(i));
        }

        memory
    }

    pub unsafe fn write_memory(&mut self, bytes: &Vec<u8>) {
        for i in 0..bytes.len() {
            *self.ptr.add(i) = bytes[i];
        }
    }

    pub unsafe fn copy_memory(&mut self, bytes: *const u8, size: usize) {
        std::ptr::copy_nonoverlapping(bytes, self.ptr, size);
    }

    pub unsafe fn fill_memory(&mut self, byte: u8, size: usize) {
        std::ptr::write_bytes(self.ptr, byte, size);
    }

    pub unsafe fn write<T>(&mut self, data: T) {
        std::ptr::write::<T>(self.ptr as *mut T, data);
    }

    pub unsafe fn read<T>(&mut self) -> T {
        std::ptr::read::<T>(self.ptr as *mut T)
    }

    pub unsafe fn write_ptr_val<T>(&mut self, offset: usize, value: T) -> bool {
        if self.ptr.is_null() {
            return false;
        }

        // [[self.ptr]+offset] = value, where [ptr] derefs ptr.
        let ptr_to_val = *(self.ptr as *const usize) + offset;
        *(ptr_to_val as *mut T) = value;
        true
    }

    pub unsafe fn read_ptr_val<T: Clone>(&mut self, offset: usize) -> Option<T> {
        if self.ptr.is_null() {
            return None;
        }

        // return = [[self.ptr]+offset], where [ptr] derefs ptr.
        let ptr_to_val = *(self.ptr as *const usize) + offset;
        return Some((*(ptr_to_val as *mut T)).clone());
    }

    pub unsafe fn write_multilevel_ptr_val<T>(&mut self, offsets: &Vec<usize>, value: T) -> bool {
        if self.ptr.is_null() {
            return false;
        }

        let mut base = *(self.ptr as *const usize);
        for i in 0..offsets.len() {
            if i == offsets.len() - 1 {
                let ptr_to_val = base + offsets[i];
                *(ptr_to_val as *mut T) = value;
                return true;
            } else {
                // the for loop deref our base
                base = *((base + offsets[i]) as *const usize);
            }
        }

        false
    }

    pub unsafe fn read_multilevel_ptr_val<T: Clone>(&mut self, offsets: &Vec<usize>) -> Option<T> {
        if self.ptr.is_null() {
            return None;
        }

        let mut base = *(self.ptr as *const usize);
        for i in 0..offsets.len() {
            if i == offsets.len() - 1 {
                let ptr_to_val = base + offsets[i];
                return Some((*(ptr_to_val as *mut T)).clone());
            } else {
                // the for loop deref our base
                base = *((base + offsets[i]) as *const usize);
            }
        }

        None
    }
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn test_address_new() {
        std::env::set_var("RUST_BACKTRACE", "1");

        assert_eq!(0xab as *mut u8, Address::new(0xab as *mut u8).ptr);
    }

    #[test]
    fn test_address_read_memory() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0x12345678;

        unsafe {
            assert_eq!(
                vec![0x78, 0x56, 0x34, 0x12],
                Address::new(std::ptr::addr_of!(n) as *mut u8).read_memory(4)
            );
        }
    }

    #[test]
    fn test_address_write_memory() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;

        unsafe {
            Address::new(std::ptr::addr_of!(n) as *mut u8)
                .write_memory(&vec![0x78, 0x56, 0x34, 0x12]);
            assert_eq!(0x12345678, n);
        }
    }

    #[test]
    fn test_address_copy_memory() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;
        let bytes = vec![0x78, 0x56, 0x34, 0x12];

        unsafe {
            Address::new(std::ptr::addr_of!(n) as *mut u8).copy_memory(bytes.as_ptr(), bytes.len());
            assert_eq!(0x12345678, n);
        }
    }

    #[test]
    fn test_address_fill_memory() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;

        unsafe {
            Address::new(std::ptr::addr_of!(n) as *mut u8).fill_memory(0x90, 3);
            assert_eq!(0xde909090, n);
        }
    }

    #[test]
    fn test_address_write() {
        let n = 0x12345678deadbeefu64;

        unsafe {
            Address::new(std::ptr::addr_of!(n) as *mut u8).write(0x90u8);
            assert_eq!(0x12345678deadbe90u64, n);

            Address::new(std::ptr::addr_of!(n) as *mut u8).write(0xbaadu16);
            assert_eq!(0x12345678deadbaadu64, n);

            Address::new(std::ptr::addr_of!(n) as *mut u8).write(0xdeadbeefu32);
            assert_eq!(0x12345678deadbeefu64, n);

            Address::new(std::ptr::addr_of!(n) as *mut u8).write(0xdeadbeef12345678u64);
            assert_eq!(0xdeadbeef12345678u64, n);
        }
    }

    #[test]
    fn test_address_read() {
        let n = 0x12345678deadbeefu64;

        unsafe {
            assert_eq!(
                0xef,
                Address::new(std::ptr::addr_of!(n) as *mut u8).read::<u8>()
            );
            assert_eq!(
                0xbeef,
                Address::new(std::ptr::addr_of!(n) as *mut u8).read::<u16>()
            ); // ok
            assert_eq!(
                0xdeadbeef,
                Address::new(std::ptr::addr_of!(n) as *mut u8).read::<u32>()
            );
            assert_eq!(
                0x12345678deadbeef,
                Address::new(std::ptr::addr_of!(n) as *mut u8).read::<u64>()
            );
        }
    }

    macro_rules! offsetof {
        ($struct: ty, $field: ident) => {{
            #[allow(invalid_value)]
            {
                let base: $struct = std::mem::MaybeUninit::<$struct>::uninit().assume_init();
                let offset = {
                    let base_ptr = &base as *const _ as *const u8;
                    let field_ptr = &base.$field as *const _ as *const u8;
                    field_ptr.offset_from(base_ptr) as usize
                };
                offset
            }
        }};
    }

    #[repr(C)]
    struct test_struct {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
    }

    #[test]
    fn test_address_write_ptr_val() {
        let obj = test_struct {
            a: 0x33,
            b: 0x9090,
            c: 0xbaadf00d,
            d: 0xdeadbeefdeadbeef,
        };
        let ptr = std::ptr::addr_of!(obj) as *mut u8;
        let ptr_to_ptr = std::ptr::addr_of!(ptr) as *mut u8;

        unsafe {
            assert_eq!(0, offsetof!(test_struct, a));
            assert_eq!(2, offsetof!(test_struct, b));
            assert_eq!(4, offsetof!(test_struct, c));
            assert_eq!(8, offsetof!(test_struct, d));

            assert_eq!(
                false,
                Address::new(std::ptr::null::<test_struct>() as *mut u8)
                    .write_ptr_val::<u8>(offsetof!(test_struct, a), 0x88)
            );

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_ptr_val::<u8>(offsetof!(test_struct, a), 0x88)
            );
            assert_eq!(0x88, obj.a);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_ptr_val::<u16>(offsetof!(test_struct, b), 0xefef)
            );
            assert_eq!(0xefef, obj.b);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr)
                    .write_ptr_val::<u32>(offsetof!(test_struct, c), 0x45454545)
            );
            assert_eq!(0x45454545, obj.c);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr)
                    .write_ptr_val::<u64>(offsetof!(test_struct, d), 0x1234567887654321)
            );
            assert_eq!(0x1234567887654321, obj.d);
        }
    }

    #[test]
    fn test_address_read_ptr_val() {
        let obj = test_struct {
            a: 0x33,
            b: 0x9090,
            c: 0xbaadf00d,
            d: 0xdeadbeefdeadbeef,
        };
        let ptr = std::ptr::addr_of!(obj) as *mut u8;
        let ptr_to_ptr = std::ptr::addr_of!(ptr) as *mut u8;

        unsafe {
            assert_eq!(0, offsetof!(test_struct, a));
            assert_eq!(2, offsetof!(test_struct, b));
            assert_eq!(4, offsetof!(test_struct, c));
            assert_eq!(8, offsetof!(test_struct, d));

            assert_eq!(
                None,
                Address::new(std::ptr::null::<test_struct>() as *mut u8)
                    .read_ptr_val::<u8>(offsetof!(test_struct, a))
            );

            assert_eq!(
                obj.a,
                Address::new(ptr_to_ptr)
                    .read_ptr_val::<u8>(offsetof!(test_struct, a))
                    .unwrap()
            );
            assert_eq!(
                obj.b,
                Address::new(ptr_to_ptr)
                    .read_ptr_val::<u16>(offsetof!(test_struct, b))
                    .unwrap()
            );
            assert_eq!(
                obj.c,
                Address::new(ptr_to_ptr)
                    .read_ptr_val::<u32>(offsetof!(test_struct, c))
                    .unwrap()
            );
            assert_eq!(
                obj.d,
                Address::new(ptr_to_ptr)
                    .read_ptr_val::<u64>(offsetof!(test_struct, d))
                    .unwrap()
            );
        }
    }

    #[repr(C)]
    struct test_struct_multilevel_inner {
        x: u32,
        y: *const u32,
        z: u32,
    }

    #[repr(C)]
    struct test_struct_multilevel {
        a: u8,
        b: u16,
        c: *const u32,
        d: *const test_struct_multilevel_inner,
        e: u64,
    }

    #[test]
    fn test_address_write_multilevel_ptr_val() {
        let v1 = 0xc0cac0ca;
        let v2 = 0xbaadf00d;

        let inner_obj = test_struct_multilevel_inner {
            x: 0x11223344,
            y: std::ptr::addr_of!(v1),
            z: 0x56565656,
        };

        let obj = test_struct_multilevel {
            a: 0x33,
            b: 0x9090,
            c: std::ptr::addr_of!(v2),
            d: std::ptr::addr_of!(inner_obj),
            e: 0xdeadbeefdeadbeef,
        };

        let ptr = std::ptr::addr_of!(obj) as *mut u8;
        let ptr_to_ptr = std::ptr::addr_of!(ptr) as *mut u8;

        unsafe {
            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u8>(
                    &vec![offsetof!(test_struct_multilevel, a)],
                    0x88
                )
            );
            assert_eq!(0x88, obj.a);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u16>(
                    &vec![offsetof!(test_struct_multilevel, b)],
                    0xefef
                )
            );
            assert_eq!(0xefef, obj.b);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u32>(
                    &vec![offsetof!(test_struct_multilevel, c), 0],
                    0x45454545
                )
            );
            assert_eq!(0x45454545, *obj.c);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u32>(
                    &vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, x)
                    ],
                    0x11111111
                )
            );
            assert_eq!(0x11111111, (*obj.d).x);
            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u32>(
                    &vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, y),
                        0
                    ],
                    0x77777777
                )
            );
            assert_eq!(0x77777777, *(*obj.d).y);
            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u32>(
                    &vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, z)
                    ],
                    0x66666666
                )
            );
            assert_eq!(0x66666666, (*obj.d).z);

            assert_eq!(
                true,
                Address::new(ptr_to_ptr).write_multilevel_ptr_val::<u64>(
                    &vec![offsetof!(test_struct_multilevel, e)],
                    0x1234567887654321
                )
            );
            assert_eq!(0x1234567887654321, obj.e);
        }
    }

    #[test]
    fn test_address_read_multilevel_ptr_val() {
        let v1 = 0xc0cac0ca;
        let v2 = 0xbaadf00d;

        let inner_obj = test_struct_multilevel_inner {
            x: 0x11223344,
            y: std::ptr::addr_of!(v1),
            z: 0x56565656,
        };

        let obj = test_struct_multilevel {
            a: 0x33,
            b: 0x9090,
            c: std::ptr::addr_of!(v2),
            d: std::ptr::addr_of!(inner_obj),
            e: 0xdeadbeefdeadbeef,
        };

        let ptr = std::ptr::addr_of!(obj) as *mut u8;
        let ptr_to_ptr = std::ptr::addr_of!(ptr) as *mut u8;

        unsafe {
            assert_eq!(
                obj.a,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u8>(&vec![offsetof!(test_struct_multilevel, a)])
                    .unwrap()
            );
            assert_eq!(
                obj.b,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u16>(&vec![offsetof!(test_struct_multilevel, b)])
                    .unwrap()
            );
            assert_eq!(
                *obj.c,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u32>(&vec![offsetof!(test_struct_multilevel, c), 0])
                    .unwrap()
            );

            assert_eq!(
                (*obj.d).x,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u32>(&vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, x)
                    ])
                    .unwrap()
            );
            assert_eq!(
                *(*obj.d).y,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u32>(&vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, y),
                        0
                    ])
                    .unwrap()
            );
            assert_eq!(
                (*obj.d).z,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u32>(&vec![
                        offsetof!(test_struct_multilevel, d),
                        offsetof!(test_struct_multilevel_inner, z)
                    ])
                    .unwrap()
            );

            assert_eq!(
                obj.e,
                Address::new(ptr_to_ptr)
                    .read_multilevel_ptr_val::<u64>(&vec![offsetof!(test_struct_multilevel, e)])
                    .unwrap()
            );
        }
    }
}
