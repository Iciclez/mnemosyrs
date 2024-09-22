use crate::address::Address;

pub trait MemoryEdit {
    fn edit(&mut self);
    fn revert(&mut self);
}

pub struct MemoryPatch {
    ptr: Address,
    replace_bytes: Vec<u8>,
    retain_bytes: Vec<u8>,
}

pub struct MemoryDataEdit<T> {
    ptr: Address,
    replace_data: T,
    retain_data: T,
}

impl MemoryPatch {
    pub fn new(ptr: Address, bytes: Vec<u8>) -> Self {
        let mut memory_patch = MemoryPatch {
            ptr: ptr,
            replace_bytes: bytes.clone(),
            retain_bytes: vec![],
        };

        unsafe {
            memory_patch.retain_bytes = memory_patch.ptr.read_memory(bytes.len());
        }

        memory_patch
    }
}

impl MemoryEdit for MemoryPatch {
    fn edit(&mut self) {
        unsafe { self.ptr.write_memory(&self.replace_bytes) }
    }

    fn revert(&mut self) {
        unsafe { self.ptr.write_memory(&self.retain_bytes) }
    }
}

impl<T: Clone> MemoryDataEdit<T> {
    pub fn new(ptr: Address, data: T) -> Self {
        let mut memory_data_edit = MemoryDataEdit::<T> {
            ptr: ptr,
            replace_data: data.clone(),
            retain_data: data.clone(),
        };

        unsafe {
            memory_data_edit.retain_data = memory_data_edit.ptr.read::<T>();
        }

        memory_data_edit
    }
}

impl<T: Clone> MemoryEdit for MemoryDataEdit<T> {
    fn edit(&mut self) {
        unsafe { self.ptr.write::<T>(self.replace_data.clone()) }
    }

    fn revert(&mut self) {
        unsafe { self.ptr.write::<T>(self.retain_data.clone()) }
    }
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn test_memory_patch_edit() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;
        let mut patch = MemoryPatch::new(
            Address::new(std::ptr::addr_of!(n) as *mut u8),
            vec![0x78, 0x56, 0x34, 0x12],
        );

        patch.edit();
        assert_eq!(0x12345678u32, n);

        patch.revert();
        patch.edit();
        assert_eq!(0x12345678u32, n);
    }

    #[test]
    fn test_memory_patch_revert() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;
        let mut patch = MemoryPatch::new(
            Address::new(std::ptr::addr_of!(n) as *mut u8),
            vec![0x78, 0x56, 0x34, 0x12],
        );

        patch.edit();
        patch.revert();
        assert_eq!(0xdeadbeefu32, n);

        patch.edit();
        patch.revert();
        assert_eq!(0xdeadbeefu32, n);
    }

    #[test]
    fn test_memory_data_edit_edit() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;
        let mut data_edit =
            MemoryDataEdit::<i32>::new(Address::new(std::ptr::addr_of!(n) as *mut u8), 0x12345678);

        data_edit.edit();
        assert_eq!(0x12345678, n);

        data_edit.revert();
        data_edit.edit();
        assert_eq!(0x12345678, n);
    }

    #[test]
    fn test_memory_data_edit_revert() {
        std::env::set_var("RUST_BACKTRACE", "1");

        let n = 0xdeadbeefu32;
        let mut data_edit =
            MemoryDataEdit::<i32>::new(Address::new(std::ptr::addr_of!(n) as *mut u8), 0x12345678);

        data_edit.edit();
        data_edit.revert();
        assert_eq!(0xdeadbeef, n);

        data_edit.edit();
        data_edit.revert();
        assert_eq!(0xdeadbeef, n);
    }
}
