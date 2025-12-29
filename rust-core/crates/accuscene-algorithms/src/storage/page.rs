//! Fixed-size page management for disk-based storage.
//!
//! Pages are the fundamental unit of I/O for disk-based storage engines.
//! Typically 4KB or 8KB in size.
//!
//! # Complexity
//! - Read: O(1) with buffer pool caching
//! - Write: O(1) with buffer pool caching
//! - Flush: O(1) per page

use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Page identifier.
pub type PageId = u64;

/// Standard page size (4KB).
pub const PAGE_SIZE: usize = 4096;

/// Page header.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageHeader {
    /// Page ID.
    pub page_id: PageId,
    /// LSN of last modification.
    pub lsn: u64,
    /// Checksum for integrity.
    pub checksum: u32,
    /// Free space offset.
    pub free_offset: u16,
    /// Number of slots.
    pub slot_count: u16,
}

impl PageHeader {
    const SIZE: usize = 24; // Fixed header size

    fn new(page_id: PageId) -> Self {
        Self {
            page_id,
            lsn: 0,
            checksum: 0,
            free_offset: Self::SIZE as u16,
            slot_count: 0,
        }
    }

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.page_id.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.lsn.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.checksum.to_le_bytes());
        bytes[20..22].copy_from_slice(&self.free_offset.to_le_bytes());
        bytes[22..24].copy_from_slice(&self.slot_count.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(AlgorithmError::PageError("Header too small".to_string()));
        }

        Ok(Self {
            page_id: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            lsn: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            checksum: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            free_offset: u16::from_le_bytes(bytes[20..22].try_into().unwrap()),
            slot_count: u16::from_le_bytes(bytes[22..24].try_into().unwrap()),
        })
    }
}

/// Page of data.
#[derive(Clone)]
pub struct Page {
    header: PageHeader,
    data: Vec<u8>,
    dirty: bool,
}

impl Page {
    /// Create a new empty page.
    pub fn new(page_id: PageId) -> Self {
        Self {
            header: PageHeader::new(page_id),
            data: vec![0u8; PAGE_SIZE],
            dirty: false,
        }
    }

    /// Get page ID.
    pub fn id(&self) -> PageId {
        self.header.page_id
    }

    /// Check if page is dirty (modified).
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark page as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark page as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get LSN.
    pub fn lsn(&self) -> u64 {
        self.header.lsn
    }

    /// Set LSN.
    pub fn set_lsn(&mut self, lsn: u64) {
        self.header.lsn = lsn;
        self.dirty = true;
    }

    /// Read data at offset.
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        if offset + buf.len() > PAGE_SIZE {
            return Err(AlgorithmError::PageError("Read beyond page".to_string()));
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    /// Write data at offset.
    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > PAGE_SIZE {
            return Err(AlgorithmError::PageError("Write beyond page".to_string()));
        }
        self.data[offset..offset + data.len()].copy_from_slice(data);
        self.dirty = true;
        Ok(())
    }

    /// Get free space available.
    pub fn free_space(&self) -> usize {
        PAGE_SIZE - self.header.free_offset as usize
    }

    /// Serialize page to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PAGE_SIZE);

        // Write header
        bytes.extend_from_slice(&self.header.to_bytes());

        // Write data
        bytes.extend_from_slice(&self.data[PageHeader::SIZE..]);

        // Ensure exact page size
        bytes.resize(PAGE_SIZE, 0);

        bytes
    }

    /// Deserialize page from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != PAGE_SIZE {
            return Err(AlgorithmError::PageError(format!(
                "Invalid page size: {}",
                bytes.len()
            )));
        }

        let header = PageHeader::from_bytes(bytes)?;
        let mut data = vec![0u8; PAGE_SIZE];
        data.copy_from_slice(bytes);

        Ok(Self {
            header,
            data,
            dirty: false,
        })
    }

    /// Compute checksum for the page.
    pub fn compute_checksum(&self) -> u32 {
        seahash::hash(&self.data) as u32
    }

    /// Verify page checksum.
    pub fn verify_checksum(&self) -> bool {
        self.header.checksum == self.compute_checksum()
    }

    /// Update checksum.
    pub fn update_checksum(&mut self) {
        self.header.checksum = self.compute_checksum();
    }
}

/// Page file manager for disk I/O.
pub struct PageFile {
    file: Arc<RwLock<File>>,
    path: PathBuf,
    page_count: Arc<RwLock<u64>>,
}

impl PageFile {
    /// Open or create a page file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;

        let metadata = file.metadata()?;
        let page_count = metadata.len() / PAGE_SIZE as u64;

        Ok(Self {
            file: Arc::new(RwLock::new(file)),
            path,
            page_count: Arc::new(RwLock::new(page_count)),
        })
    }

    /// Read a page from disk.
    pub fn read_page(&self, page_id: PageId) -> Result<Page> {
        let mut file = self.file.write();
        let offset = page_id * PAGE_SIZE as u64;

        file.seek(SeekFrom::Start(offset))?;

        let mut bytes = vec![0u8; PAGE_SIZE];
        file.read_exact(&mut bytes)?;

        Page::from_bytes(&bytes)
    }

    /// Write a page to disk.
    pub fn write_page(&self, page: &mut Page) -> Result<()> {
        page.update_checksum();

        let mut file = self.file.write();
        let offset = page.id() * PAGE_SIZE as u64;

        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&page.to_bytes())?;

        page.mark_clean();

        // Update page count if necessary
        let mut page_count = self.page_count.write();
        if page.id() >= *page_count {
            *page_count = page.id() + 1;
        }

        Ok(())
    }

    /// Allocate a new page.
    pub fn allocate_page(&self) -> Result<Page> {
        let mut page_count = self.page_count.write();
        let page_id = *page_count;
        *page_count += 1;

        Ok(Page::new(page_id))
    }

    /// Flush all pending writes.
    pub fn flush(&self) -> Result<()> {
        self.file.write().sync_all()?;
        Ok(())
    }

    /// Get number of pages.
    pub fn page_count(&self) -> u64 {
        *self.page_count.read()
    }
}

impl Clone for PageFile {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            path: self.path.clone(),
            page_count: self.page_count.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_page_read_write() {
        let mut page = Page::new(0);
        let data = b"Hello, World!";

        page.write_at(100, data).unwrap();
        assert!(page.is_dirty());

        let mut buf = vec![0u8; data.len()];
        page.read_at(100, &mut buf).unwrap();

        assert_eq!(data, buf.as_slice());
    }

    #[test]
    fn test_page_serialization() {
        let mut page = Page::new(42);
        page.write_at(100, b"test data").unwrap();
        page.update_checksum();

        let bytes = page.to_bytes();
        let page2 = Page::from_bytes(&bytes).unwrap();

        assert_eq!(page.id(), page2.id());
        assert!(page2.verify_checksum());
    }

    #[test]
    fn test_page_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let page_file = PageFile::open(temp_file.path()).unwrap();

        let mut page = page_file.allocate_page().unwrap();
        page.write_at(50, b"test").unwrap();

        page_file.write_page(&mut page).unwrap();

        let loaded_page = page_file.read_page(page.id()).unwrap();
        assert_eq!(page.id(), loaded_page.id());
    }
}
