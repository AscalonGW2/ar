use std::{collections::BTreeMap, fs::OpenOptions, path::Path};

use bytemuck::{Pod, Zeroable};
use memmap2::Mmap;

pub struct Archive {
    mmap: Mmap,
    index: BTreeMap<u32, MftEntry>,
}

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };

        let an_header: &AnHeader = bytemuck::from_bytes(&mmap[..std::mem::size_of::<AnHeader>()]);
        assert_eq!(an_header.version, 151);
        assert_eq!(&an_header.magic, b"AN\x1A");

        let mft = &mmap[an_header.mft_offset as usize..][..an_header.mft_size as usize];
        let (mft_header, mft_entries) = mft.split_at(std::mem::size_of::<MftHeader>());
        let mft_header: &MftHeader = bytemuck::from_bytes(mft_header);
        assert_eq!(&mft_header.magic, b"Mft\x1A");

        let mft_entries: &[MftEntry] = bytemuck::cast_slice(mft_entries);

        let an_mft_entry = &mft_entries[0];
        assert_eq!(an_mft_entry.size as u32, 40);
        assert_eq!(an_mft_entry.extra_bytes as u16, 0);
        assert_eq!(an_mft_entry.flags, 1 | 2);

        let index_mft_entry = &mft_entries[1];
        assert_eq!(index_mft_entry.extra_bytes as u16, 0);
        assert_eq!(index_mft_entry.flags, 1 | 2);
        let index_entries: &[IndexEntry] = bytemuck::cast_slice(
            &mmap[index_mft_entry.offset as usize..][..index_mft_entry.size as usize],
        );
        let mut index = BTreeMap::new();
        for index_entry in index_entries {
            if index_entry.file_id == 0 {
                continue;
            }
            index.insert(
                index_entry.file_id,
                mft_entries[index_entry.mft_index as usize - 1],
            );
        }

        let mft_mft_entry = &mft_entries[2];
        assert_eq!(an_header.mft_offset as u64, mft_mft_entry.offset as u64);
        assert_eq!(an_header.mft_size as u32, mft_mft_entry.size as u32);
        assert_eq!(mft_mft_entry.extra_bytes as u16, 0);
        assert_eq!(mft_mft_entry.flags, 1 | 2);

        Self { mmap, index }
    }

    pub fn get(&self, file_id: u32) -> Option<Vec<u8>> {
        let Some(entry) = self.index.get(&file_id) else {
            return None;
        };

        let mut data = &self.mmap[entry.offset as usize..][..entry.size as usize];
        if entry.extra_bytes != 8 {
            return None;
        }

        let mut uncompressed_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let mut uncompressed_data = vec![0; uncompressed_size as usize];
        unsafe {
            compression_inflateDatFileBuffer(
                entry.size,
                data.as_ptr(),
                &mut uncompressed_size,
                uncompressed_data.as_mut_ptr(),
            );
        }
        return Some(uncompressed_data);
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct AnHeader {
    version: u8,
    magic: [u8; 3],
    _2: u32,
    _3: u32,
    _4: u32,
    crc: u32,
    _6: u32,
    mft_offset: u64,
    mft_size: u32,
    flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct MftHeader {
    magic: [u8; 4],
    _1: u64,
    _2: u32,
    _3: u32,
    _4: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct MftEntry {
    offset: u64,
    size: u32,
    extra_bytes: u16,
    flags: u8,
    _4: u8,
    _5: u32,
    crc: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct IndexEntry {
    file_id: u32,
    mft_index: u32,
}

#[link(name = "gw2dattools", kind = "static")]
extern "C" {
    fn compression_inflateDatFileBuffer(
        iInputSize: u32,
        iInputTab: *const u8,
        ioOutputSize: *mut u32,
        ioOutputTab: *mut u8,
    ) -> *mut u8;
}
