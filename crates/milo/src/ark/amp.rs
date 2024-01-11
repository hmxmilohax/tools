#[derive(Clone, Copy)]
struct AmpFileEntry {
    offset: u32,
    file_name_idx: u32,
    folder_name_idx: u32,
    size: u32,
    inflated_size: u32,
}

pub struct AmpArchive {
    version: u32,
    entries: Vec<AmpFileEntry>,
    str_table_size: u32,
    string_table: Vec<String>,
    string_idx_count: u32,
    string_idx_entries: Vec<u32>
}

impl Clone for AmpArchive {
    fn clone(&self) -> Self {
        let mut new_entries: Vec<AmpFileEntry> = vec![];
        for entry in &self.entries {
            new_entries.push(*entry);
        }
        let mut new_strtbl: Vec<String> = vec![];
        for string in &self.string_table {
            new_strtbl.push(string.clone());
        }
        let mut new_stridxs: Vec<u32> = vec![];
        for idx in &self.string_idx_entries {
            new_stridxs.push(*idx);
        }
        Self {
            version: self.version,
            entries: new_entries,
            str_table_size: self.str_table_size,
            string_table: new_strtbl,
            string_idx_count: self.string_idx_count,
            string_idx_entries: new_stridxs
        }
    }
}
