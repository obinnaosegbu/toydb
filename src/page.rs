pub const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageId(pub u64);

pub struct Page {
    id: PageId,
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(id: PageId) -> Self {
        Self {
            id,
            data: [0u8; PAGE_SIZE],
        }
    }

    pub fn from_data(id: PageId, data: [u8; PAGE_SIZE]) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> PageId {
        self.id
    }

    pub fn data(&self) -> &[u8; PAGE_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        &mut self.data
    }
}
