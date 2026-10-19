use crate::page::PageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordId {
    page_id: PageId,
    slot_id: u32,
}

impl RecordId {
    pub fn new(page_id: PageId, slot_id: u32) -> Self {
        Self { page_id, slot_id }
    }

    pub fn page_id(&self) -> PageId {
        self.page_id
    }

    pub fn slot_id(&self) -> u32 {
        self.slot_id
    }
}
