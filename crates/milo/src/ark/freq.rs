use crate::traits::{Load, Port};
use crate::fio;
use std::fmt::Display;
use std::fs::File;
use std::error::Error;

#[derive(Clone, Copy)]
struct FreqFileEntry { // 24 bytes
    unknown: u32, // Path name hash?
    file_name_offset: u32,
    folder_name_index: u16,
    block_offset: u16,
    block: u32, // Use block * block_size + block_offset to get file position
    file_size: u32,
    inflated_size: u32, // Same as file size if not compressed
    fake_file_offset: u32 // = (block * 2048) + block_offset;
}

impl Load for FreqFileEntry {
    fn load(&mut self, f: &mut File, _: u32) -> Result<(), Box<dyn Error>> {
        self.unknown = fio::read_u32(f, true)?;
        self.file_name_offset = fio::read_u32(f, true)?;
        self.folder_name_index = fio::read_u16(f, true)?;
        self.block_offset = fio::read_u16(f, true)?;
        self.block = fio::read_u32(f, true)?;
        self.file_size = fio::read_u32(f, true)?;
        self.inflated_size = fio::read_u32(f, true)?;
        self.fake_file_offset = (self.block * 2048) + self.block_offset as u32;
        Ok(())
    }
}

impl Display for FreqFileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Unknown value (possibly a pathname hash?): {}", self.unknown))?;
        f.write_fmt(format_args!("File name offset: {}", self.file_name_offset))?;
        f.write_fmt(format_args!("Folder name index: {}", self.folder_name_index))?;
        f.write_fmt(format_args!("Block offset: {}", self.block_offset))?;
        f.write_fmt(format_args!("Block #: {}", self.block))?;
        f.write_fmt(format_args!("File offset (calculated): {}", self.fake_file_offset))?;
        f.write_fmt(format_args!("File size: {}", self.file_size))?;
        f.write_fmt(format_args!("Inflated filesize: {}", self.inflated_size))?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct FreqArchive {
    magic: u32, // technically a char[4] but 0x204B5241 is easier to check
    version: u32,
    file_entry_offset: u32, // Always 256
    file_entry_count: u32,
    folder_entry_offset: u32,
    folder_entry_count: u32,
    string_table_offset: u32,
    string_count: u32,
    total_hdr_size: u32, // Size of header + string offsets + string table
    block_size: u32, // Used for padding, always 2048?
}

impl FreqArchive {
    pub fn new() -> Self {
        Self {
            magic: 0x204B5241,
            version: 0,
            file_entry_offset: 256,
            file_entry_count: 0,
            folder_entry_offset: 0,
            folder_entry_count: 0,
            string_table_offset: 0,
            string_count: 0,
            total_hdr_size: 40,
            block_size: 2048
        }
    }
}

impl Load for FreqArchive {
    fn load(&mut self, f: &mut File, _: u32) -> Result<(), Box<dyn Error>> {
        self.magic = fio::read_u32(f, true)?;
        self.version = fio::read_u32(f, true)?;
        self.file_entry_offset = fio::read_u32(f, true)?;
        self.file_entry_count = fio::read_u32(f, true)?;
        self.folder_entry_offset = fio::read_u32(f, true)?;
        self.folder_entry_count = fio::read_u32(f, true)?;
        self.string_table_offset = fio::read_u32(f, true)?;
        self.string_count = fio::read_u32(f, true)?;
        self.total_hdr_size = fio::read_u32(f, true)?;
        self.block_size = fio::read_u32(f, true)?;
        Ok(())
    }
}

impl Display for FreqArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Magic value: {:#010X} \n", self.magic))?;
        f.write_fmt(format_args!("Should be    0x004B5241.\n"))?;
        f.write_fmt(format_args!("Version: {}\n", self.version))?;
        f.write_fmt(format_args!("File entry offset: {}\n", self.file_entry_offset))?;
        f.write_fmt(format_args!("File entry count: {}\n", self.file_entry_count))?;
        f.write_fmt(format_args!("Folder entry offset: {}\n", self.folder_entry_offset))?;
        f.write_fmt(format_args!("Folder entry count: {}\n", self.folder_entry_count))?;
        f.write_fmt(format_args!("String table offset: {}\n", self.string_table_offset))?;
        f.write_fmt(format_args!("String count: {}\n", self.string_count))?;
        f.write_fmt(format_args!("Total header size (includes string table): {}\n", self.total_hdr_size))?;
        f.write_fmt(format_args!("Block size: {}\n", self.block_size))?;
        Ok(())
    }
}
