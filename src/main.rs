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

    
    // PageLayout capacity and persistence tests
    let mut layout = PageLayout::new();
    let mut layout_page = Page::new(PageId(3));

    layout.write_header_to_page(&mut layout_page);

    let record = [7u8; 92];
    let mut expected_records: Vec<Vec<u8>> = Vec::new();

    // Fill the page until a record no longer fits.
    loop {
        match layout.insert_record(&mut layout_page, &record) {
            Ok(slot_id) => {
                assert_eq!(slot_id, expected_records.len());

                let expected = record.to_vec();
                let actual = layout.get_record(&layout_page, slot_id);

                assert_eq!(actual, expected);
                expected_records.push(expected);
            }
            Err(error) => {
                println!("Insertion rejected: {error}");
                break;
            }
        }
    }

    // Snapshot the complete page before another failed insertion.
    let page_before_failure = *layout_page.data();
    let header_before_failure = (
        layout.page_header.num_slots,
        layout.page_header.free_space_start,
        layout.page_header.free_space_end,
        layout.page_header.reserved,
    );

    let failure = layout.insert_record(&mut layout_page, &record);
    assert!(failure.is_err());

    // A failed insertion must not change any page bytes or header fields.
    assert_eq!(*layout_page.data(), page_before_failure);
    assert_eq!(
        (
            layout.page_header.num_slots,
            layout.page_header.free_space_start,
            layout.page_header.free_space_end,
            layout.page_header.reserved,
        ),
        header_before_failure
    );

    // All existing records must still be readable.
    for (slot_id, expected) in expected_records.iter().enumerate() {
        assert_eq!(layout.get_record(&layout_page, slot_id), *expected);
    }

    // A record too large for an empty page must be rejected.
    let mut empty_layout = PageLayout::new();
    let mut empty_page = Page::new(PageId(4));
    empty_layout.write_header_to_page(&mut empty_page);

    let oversized_record = vec![0u8; 4096];
    let empty_page_before = *empty_page.data();

    assert!(empty_layout
        .insert_record(&mut empty_page, &oversized_record)
        .is_err());

    assert_eq!(*empty_page.data(), empty_page_before);
    assert_eq!(empty_layout.page_header.num_slots, 0);

    // A smaller record should still fit after the failed insertion.
    let small_record = [50, 67, 19, 89, 90, 12, 68];

    let small_slot = layout
        .insert_record(&mut layout_page, &small_record)
        .expect("The smaller record should fit");

    assert_eq!(small_slot, expected_records.len());

    expected_records.push(small_record.to_vec());

    // Write to disk and read back into a new Page.
    disk_manager.write(&layout_page)?;

    let retrieved_page = disk_manager.read(PageId(3))?;
    let restored_layout = PageLayout::from_page(&retrieved_page);

    assert_eq!(
        restored_layout.page_header.num_slots as usize,
        expected_records.len()
    );
    assert_eq!(restored_layout.slots.len(), expected_records.len());

    for (slot_id, expected) in expected_records.iter().enumerate() {
        let actual = restored_layout.get_record(&retrieved_page, slot_id);
        assert_eq!(&actual, expected);
    }

    println!(
        "PageLayout tests passed: {} records survived the disk round trip.",
        expected_records.len()
    );



    let mut buffer_pool = BufferPoolManager::new(disk_manager, 2);

    let fetch_page = buffer_pool
        .fetch_page(page.id())
        .expect("Page should be fetched");

    println!("{:?}", &fetch_page.data()[..5]);

    let unpin = buffer_pool.unpin_page(page.id(), false);
    println!("{}", unpin);

    let fetch_page_again = buffer_pool
        .fetch_page(page1.id())
        .expect("Page should be fetched again");

    println!("{:?}", &fetch_page_again.data()[..5]);

    let unpin: bool = buffer_pool.unpin_page(page1.id(), false);
    println!("{}", unpin);

    let fetch_page2 = buffer_pool
        .fetch_page(page2.id())
        .expect("Page2 should be fetched");

    println!("{:?}", &fetch_page2.data()[..5]);

    let fetch_page = buffer_pool
        .fetch_page(page1.id())
        .expect("Page should be fetched");

    println!("{:?}", &fetch_page.data()[..5]);

    Ok(())
}