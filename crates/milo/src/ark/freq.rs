use crate::traits;
use std::fmt::Display;

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
struct FreqArchive {
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

impl Display for FreqArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Magic value: {}There should be a space. If there isn't, this is a bad FreqArchive.", self.magic))?;
        f.write_fmt(format_args!("Version: {}", self.version))?;
        f.write_fmt(format_args!("File entry offset: {}", self.file_entry_offset))?;
        f.write_fmt(format_args!("File entry count: {}", self.file_entry_count))?;
        f.write_fmt(format_args!("Folder entry offset: {}", self.folder_entry_offset))?;
        f.write_fmt(format_args!("Folder entry count: {}", self.folder_entry_count))?;
        f.write_fmt(format_args!("String table offset: {}", self.string_table_offset))?;
        f.write_fmt(format_args!("String count: {}", self.string_count))?;
        f.write_fmt(format_args!("Total header size (includes string table): {}", self.total_hdr_size))?;
        f.write_fmt(format_args!("Block size: {}", self.block_size))?;
        Ok(())
    }
}
