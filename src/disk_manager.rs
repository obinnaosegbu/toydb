use crate::page::{PAGE_SIZE, Page, PageId};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;

pub struct DiskManager {
    file: File,
}

impl DiskManager {
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn write(&self, page: &Page) -> std::io::Result<()> {
        let offset: u64 = page.id().0 * PAGE_SIZE as u64;
        self.file.write_all_at(page.data(), offset)?;
        Ok(())
    }

    pub fn read(&self, page_id: PageId) -> std::io::Result<Page> {
        let offset = page_id.0 * PAGE_SIZE as u64;
        let mut buffer: [u8; PAGE_SIZE] = [0u8; PAGE_SIZE];
        self.file.read_exact_at(&mut buffer, offset)?;
        Ok(Page::from_data(page_id, buffer))
    }

    pub fn allocate_page(&self) -> std::io::Result<Page> {
        let file_size = self.file.metadata()?.len();

        if file_size % PAGE_SIZE as u64 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "database file size is not page-alligned",
            ));
        }

        let page_id = PageId(file_size / PAGE_SIZE as u64);
        let page = Page::new(page_id);

        self.write(&page)?;

        Ok(page)
    }
}
