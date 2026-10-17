use std::collections::HashMap;

use crate::page::{Page, PageId};
use crate::disk_manager::DiskManager;

#[derive(Clone, Copy, Debug)]
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

    pub fn new_page(&mut self) -> Option<PageId>{
        let page = self.disk_manager.allocate_page().ok()?;
        let page_id = page.id();

        let frame_id = self.find_free_frame()?;

        if !self.evict_frame(frame_id) {
            return None;
        }

        let frame = &mut self.frames[frame_id.0 as usize];
        frame.page = Some(page);
        frame.dirty = false;
        frame.pin_count = 1;

        self.page_table.insert(page_id, frame_id);

        Some(page_id)
    }

    pub fn fetch_page_mut(&mut self, page_id: PageId) -> Option<&mut Page>{
        // Page is already loaded into the buffer pool.
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame = &mut self.frames[frame_id.0 as usize];
            
            frame.pin_count += 1;

            return frame.page.as_mut();
        } 
        
        // Page is not currently in memory.
        let frame_id = self.find_free_frame()?;

        if !self.evict_frame(frame_id){
            return None;
        }

        let page = self.disk_manager.read(page_id).ok()?;
        let frame = &mut self.frames[frame_id.0 as usize];

        frame.page = Some(page);
        frame.dirty = false;
        frame.pin_count += 1;

        self.page_table.insert(page_id, frame_id);
        
        frame.page.as_mut()
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Option<&Page>{
        // Page is already loaded into the buffer pool.
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame = &mut self.frames[frame_id.0 as usize];
            
            frame.pin_count += 1;

            return frame.page.as_ref();
        } 
        
        // Page is not currently in memory.
        let frame_id = self.find_free_frame()?;

        if !self.evict_frame(frame_id){
            return None;
        }

        let page = self.disk_manager.read(page_id).ok()?;
        let frame = &mut self.frames[frame_id.0 as usize];

        frame.page = Some(page);
        frame.dirty = false;
        frame.pin_count += 1;

        self.page_table.insert(page_id, frame_id);
        
        frame.page.as_ref()
    }

    fn find_free_frame(&self) -> Option<FrameId>{
        let mut frame_id =self.frames
            .iter()
            .find(|frame| frame.page.is_none())
            .map(|frame| frame.id);
        
        if frame_id.is_none(){
            frame_id = self.find_evictable_frame();
        }

        frame_id
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

    fn find_evictable_frame(&self) -> Option<FrameId>{
        self.frames
            .iter()
            .find(|frame| frame.pin_count == 0)
            .map(|frame| frame.id)
    }

    fn evict_frame(&mut self, frame_id: FrameId) -> bool {
        let frame = &mut self.frames[frame_id.0 as usize];

        if let Some(page) = &frame.page {
            if frame.dirty {
                let Ok(()) = self.disk_manager.write(page) else {
                    return false;
                };
            }
    
            self.page_table.remove(&page.id());
        }

        frame.page = None;
        frame.dirty = false;
        frame.pin_count = 0;

        true
    }

    pub fn flush_page(&mut self, page_id: PageId) -> bool {
        // 1. Look up the page Id from the page table.
        let Some(&frame_id) = self.page_table.get(&page_id) else {
            return false;
        };

        // 2. Fetch the corresponding frame from the memory.
        let frame = &mut self.frames[frame_id.0 as usize];

        // 3. If frame is dirty write it back to the disk.
        if frame.dirty {
            if let Some(page) = &frame.page {
                // Attempt to write the page to disk.
                let Ok(()) = self.disk_manager.write(page) else {
                    return false; // Return false if the disk write fails.
                };

                // Clear the dirty flag if the write succeeded
                frame.dirty = false;
            } else {
                // Edge case: Frame is marked dirty but contains no page data.
                return false;
            }
        }

        // 4. Return true if the page was successfuly flushed (or wasn't dirty).
        true
    }

    pub fn flush_all_pages(&mut self) -> bool {
        // 1. Collect all page ids currently in the buffer pool.
        let frame_ids: Vec<FrameId> = self.page_table.values().copied().collect();
        
        // 2. Iterate through each frames to check for dirty pages.
        for frame_id in frame_ids {
            let frame = &mut self.frames[frame_id.0 as usize];

            // Check if the frame is dirty and currently has a page.
            if frame.dirty && frame.page.is_some() {
                if let Some(page) = &frame.page {
                    // Attempt to write the page to disk.
                    let Ok(()) = self.disk_manager.write(page) else {
                        return false; // Return false if the disk write fails.
                    };

                    // Clear the dirty flag if the write succeeded
                    frame.dirty = false;
                }
            }
        }

        true
    }
}