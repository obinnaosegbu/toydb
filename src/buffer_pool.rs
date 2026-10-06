use std::collections::HashMap;

use crate::page::{Page, PageId};
use crate::disk_manager::DiskManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameId(pub u64);

struct Frame{
    id: FrameId,
    page: Option<Page>,
    dirty: bool,
    pin_count: u8,
}

pub struct BufferPoolManager{
    frames: Vec<Frame>,
    page_table: HashMap<PageId, FrameId>,
    disk_manager: DiskManager,
}

impl BufferPoolManager {
    pub fn new(disk_manager: DiskManager, size: u64) -> Self {
        let mut frames = Vec::with_capacity(size as usize);

        for index in 0..size {
            frames.push(Frame{
                id: FrameId(index), 
                page: None, 
                dirty: false, 
                pin_count: 0,
            });
        }

        Self { 
            frames, 
            page_table:HashMap::new(), 
            disk_manager,
        }
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Option<&Page>{
        // Page is already loaded into the buffer pool.
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame = &mut self.frames[frame_id.0 as usize];
            
            frame.pin_count += 1;

            println!("From pool");
            return frame.page.as_ref();
        } 
        
        // Page is not currently in memory.
        let frame_id = self.find_free_frame()?;
        let page = self.disk_manager.read(page_id).ok()?;
        let frame = &mut self.frames[frame_id.0 as usize];

        frame.page = Some(page);
        frame.dirty = false;
        frame.pin_count += 1;

        self.page_table.insert(page_id, frame_id);
        
        println!("From disk");
        frame.page.as_ref()
    }

    fn find_free_frame(&self) -> Option<FrameId>{
        self.frames
            .iter()
            .find(|frame| frame.page.is_none())
            .map(|frame| frame.id)
    }

    pub fn unpin_page(&mut self, page_id: PageId, dirty: bool) -> bool {
        let Some(&frame_id) = self.page_table.get(&page_id) else{
            return false;
        };

        let frame = &mut self.frames[frame_id.0 as usize];

        if frame.pin_count == 0 {
            return false;
        }

        frame.pin_count -= 1;

        if dirty { frame.dirty = true };

        true
    }
}