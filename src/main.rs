use page::{Page, PageId};
use disk_manager::DiskManager;
use buffer_pool::BufferPoolManager;

mod page;
mod disk_manager;
mod buffer_pool;

fn main() -> std::io::Result<()> {
    let mut page = Page::new(PageId(0));
    page.data_mut()[0] = 10;
    page.data_mut()[1] = 20;
    page.data_mut()[2] = 30;
    page.data_mut()[3] = 40;
    page.data_mut()[4] = 50;

    let disk_manager = DiskManager::open("database.db")?;
    disk_manager.write(&page)?;
    
    let mut buffer_pool = BufferPoolManager::new(disk_manager, 4);

    let fetch_page = buffer_pool.fetch_page(page.id()).expect("Page should fected");
    println!("{:?}", &fetch_page.data()[..5]);


    let fetch_page_again = buffer_pool.fetch_page(page.id()).expect("Page should fected again");
    println!("{:?}", &fetch_page_again.data()[..5]);
    

    let unpin = buffer_pool.unpin_page(page.id(), false);
    println!("{}", unpin);
    
    Ok(())
}
