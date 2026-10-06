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

    let mut page1 = Page::new(PageId(1));
    page1.data_mut()[0] = 20;
    page1.data_mut()[1] = 40;
    page1.data_mut()[2] = 60;
    page1.data_mut()[3] = 80;
    page1.data_mut()[4] = 100;

    let mut page2 = Page::new(PageId(2));
    page2.data_mut()[0] = 30;
    page2.data_mut()[1] = 60;
    page2.data_mut()[2] = 90;
    page2.data_mut()[3] = 120;
    page2.data_mut()[4] = 150;

    let disk_manager = DiskManager::open("database.db")?;
    disk_manager.write(&page)?;
    disk_manager.write(&page1)?;
    disk_manager.write(&page2)?;
    
    let mut buffer_pool = BufferPoolManager::new(disk_manager, 2);

    let fetch_page = buffer_pool.fetch_page(page.id()).expect("Page should fected");
    println!("{:?}", &fetch_page.data()[..5]);

    let unpin = buffer_pool.unpin_page(page.id(), false);
    println!("{}", unpin);


    let fetch_page_again = buffer_pool.fetch_page(page1.id()).expect("Page should fected again");
    println!("{:?}", &fetch_page_again.data()[..5]);
    

    let unpin = buffer_pool.unpin_page(page1.id(), false);
    println!("{}", unpin);

    let fetch_page2 = buffer_pool.fetch_page(page2.id()).expect("Page2 should fected");
    println!("{:?}", &fetch_page2.data()[..5]);
    
    let fetch_page = buffer_pool.fetch_page(page1.id()).expect("Page should fected");
    println!("{:?}", &fetch_page.data()[..5]);
    Ok(())
}
