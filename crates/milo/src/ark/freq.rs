#[derive(Clone, Copy)]
struct FreqFileEntry { // 24 bytes
    unknown: u32, // Path name hash?
    file_name_offset: u32,
    folder_name_index: u16,
    block_offset: u16,
    block: u32, // Use block * block_size + block_offset to get file position
    file_size: u32,
    inflated_size: u32, // Same as file size if not compressed
    file_offset: u32 // = (block * 2048) + block_offset;
}
