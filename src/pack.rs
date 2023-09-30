use std::marker::PhantomData;

use bytemuck::Pod;

#[repr(C, packed)]
struct Array<T> {
    count: u32,
    offset: u64,
    _phantom: PhantomData<T>,
}

impl<T: Pod> Array<T> {
    pub fn as_slice(&self) -> &[T] {
        bytemuck::cast_slice(unsafe {
            std::slice::from_raw_parts(
                (self.offset as *const u8).offset(self.offset as isize),
                self.count as usize * std::mem::size_of::<T>(),
            )
        })
    }
}

#[repr(C, packed)]
struct Filename {
    offset: u64,
}

#[repr(C, packed)]
struct Ref<T> {
    offset: u64,
    _phantom: PhantomData<T>,
}

impl<T: Pod> Ref<T> {
    pub fn as_ref(&self) -> &T {
        bytemuck::from_bytes(unsafe {
            std::slice::from_raw_parts(
                (self.offset as *const u8).offset(self.offset as isize),
                std::mem::size_of::<T>(),
            )
        })
    }
}

#[repr(C, packed)]
struct WcharRef {
    offset: u64,
}

impl WcharRef {
    pub fn as_string(&self) -> String {
        "".to_string()
    }
}

#[repr(C, packed)]
struct CharRef {
    offset: u64,
}

impl CharRef {
    pub fn as_string(&self) -> String {
        "".to_string()
    }
}
