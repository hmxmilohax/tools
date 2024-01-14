use std::fmt::{Formatter, Display};

use crate::traits::Load;
use crate::fio;

#[derive(Clone, Copy)]
struct AmpFileEntry {
    offset: u32,
    file_name_idx: u32,
    folder_name_idx: u32,
    size: u32,
    inflated_size: u32,
}

impl AmpFileEntry {
    pub fn new() -> Self {
        Self {
            offset: 0,
            file_name_idx: 0,
            folder_name_idx: 0,
            size: 0,
            inflated_size: 0,
        }
    }
}

impl Load for AmpFileEntry {
    fn load(&mut self, f: &mut std::fs::File, ver: u32) -> Result<(), Box<dyn std::error::Error>> {
        if ver != 1 {self.offset = fio::read_u32(f, true)?;}
        self.file_name_idx = fio::read_u32(f, true)?;
        self.folder_name_idx = fio::read_u32(f, true)?;
        if ver == 1 {self.offset = fio::read_u32(f, true)?;}
        self.size = fio::read_u32(f, true)?;
        self.inflated_size = fio::read_u32(f, true)?;
        Ok(())
    }
}

impl Display for AmpFileEntry {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.write_fmt(format_args!("Offset: {}", self.offset))?;
        fmt.write_fmt(format_args!("File name index: {}", self.file_name_idx))?;
        fmt.write_fmt(format_args!("Folder name index: {}", self.folder_name_idx))?;
        fmt.write_fmt(format_args!("Size: {}", self.size))?;
        fmt.write_fmt(format_args!("Inflated size: {}", self.inflated_size))?;
        Ok(())
    }
}

pub struct AmpArchive {
    version: u32,
    entry_ct: u32,
    entries: Vec<AmpFileEntry>,
    str_table_size: u32,
    string_table: Vec<String>,
    string_idx_count: u32,
    string_idx_entries: Vec<u32>
}

impl AmpArchive {
    pub fn new() -> Self {
        Self {
            version: 0,
            entry_ct: 0,
            entries: vec![],
            str_table_size: 0,
            string_table: vec![],
            string_idx_count: 0,
            string_idx_entries: vec![]
        }
    }
}

impl Load for AmpArchive {
    fn load(&mut self, f: &mut std::fs::File, _: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.version = fio::read_u32(f, true)?;
        self.entry_ct = fio::read_u32(f, true)?;
        for _ in 0..self.entry_ct {
            let mut ent = AmpFileEntry::new();
            ent.load(f, self.version)?;
            self.entries.push(ent);
        }
        self.str_table_size = fio::read_u32(f, true)?;
        for _ in 0..self.str_table_size {
            let st = fio::readstr(f)?;
            self.string_table.push(st);
        }
        self.string_idx_count = fio::read_u32(f, true)?;
        for _ in 0..self.string_idx_count {
            let idx = fio::read_u32(f, true)?;
            self.string_idx_entries.push(idx);
        }
        Ok(())
    }
}

impl Display for AmpArchive {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.write_fmt(format_args!("Version: {}", self.version))?;
        fmt.write_str("BEGIN ENTRIES")?;
        for ent in self.entries.clone() {
            ent.fmt(fmt)?;
        }
        fmt.write_str("END ENTRIES")?;
        fmt.write_fmt(format_args!("String table size: {}", self.str_table_size))?;
        for i in 0..self.str_table_size {
            fmt.write_fmt(format_args!("Entry {i}: {}", self.string_table[i as usize]))?;
        }
        fmt.write_fmt(format_args!("String index count: {}", self.string_idx_count))?;
        for i in 0..self.string_idx_entries.len() {
            fmt.write_fmt(format_args!("Index {i}: {}", self.string_idx_entries[i]))?;
        }
        Ok(())
    }
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
            entry_ct: self.entry_ct,
            entries: new_entries,
            str_table_size: self.str_table_size,
            string_table: new_strtbl,
            string_idx_count: self.string_idx_count,
            string_idx_entries: new_stridxs
        }
    }
}
