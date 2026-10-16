use page::{Page, PageId};
use disk_manager::DiskManager;
use buffer_pool::BufferPoolManager;
use page_layout::PageLayout;

mod page;
mod disk_manager;
mod buffer_pool;
mod page_layout;
mod btree;

fn main() -> std::io::Result<()> {

    // let disk_manager = DiskManager::open("database.db")?;

    // let buffer_pool = BufferPoolManager::new(disk_manager, 4);

    Ok(())
}