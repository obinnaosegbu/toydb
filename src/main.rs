use buffer_pool::BufferPoolManager;
use disk_manager::DiskManager;
use page::{Page, PageId};
use page_layout::PageLayout;

mod btree;
mod buffer_pool;
mod disk_manager;
mod page;
mod page_layout;

fn main() -> std::io::Result<()> {
    // let disk_manager = DiskManager::open("database.db")?;

    // let buffer_pool = BufferPoolManager::new(disk_manager, 4);

    Ok(())
}
