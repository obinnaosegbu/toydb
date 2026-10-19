use crate::page::{PAGE_SIZE, Page};

const HEADER_SIZE: usize = 16;
const SLOT_SIZE: usize = 8;

#[derive(Debug)]
pub enum PageLayoutError {
    RecordTooLarge,
    InsufficientSpace,
}

impl std::fmt::Display for PageLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordTooLarge => write!(f, "record is too large to fit in an empty page"),
            Self::InsufficientSpace => write!(f, "insufficient free space in page"),
        }
    }
}

pub struct PageLayout {
    pub page_header: PageHeader,
    pub slots: Vec<Slot>,
}
pub struct PageHeader {
    pub num_slots: u32,
    pub free_space_start: u32,
    pub free_space_end: u32,
    pub reserved: u32,
}

pub struct Slot {
    offset: u32,
    length: u32,
}

impl PageHeader {
    fn serialize(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];

        bytes[0..4].copy_from_slice(&self.num_slots.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.free_space_start.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.free_space_end.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.reserved.to_le_bytes());

        bytes
    }

    fn deserialize(bytes: [u8; 16]) -> Self {
        Self {
            num_slots: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            free_space_start: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            free_space_end: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            reserved: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
        }
    }
}

impl Slot {
    fn serialize(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];

        bytes[0..4].copy_from_slice(&self.offset.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.length.to_le_bytes());

        bytes
    }

    fn deserialize(bytes: [u8; 8]) -> Self {
        Self {
            offset: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            length: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        }
    }
}

impl PageLayout {
    pub fn new() -> Self {
        let page_header = PageHeader {
            num_slots: 0,
            free_space_start: 16,
            free_space_end: 4096,
            reserved: 0,
        };

        let slots: Vec<Slot> = Vec::new();

        Self { page_header, slots }
    }

    pub fn write_header_to_page(&self, page: &mut Page) {
        let bytes: [u8; 16] = self.page_header.serialize();

        page.data_mut()[0..16].copy_from_slice(&bytes);
    }

    fn slot_offset(slot_id: usize) -> usize {
        (slot_id * SLOT_SIZE) + HEADER_SIZE
    }

    fn write_slot_to_page(&self, page: &mut Page, slot_id: usize) {
        let slot = &self.slots[slot_id];
        let bytes = slot.serialize();
        let offset = Self::slot_offset(slot_id);

        page.data_mut()[offset..offset + 8].copy_from_slice(&bytes);
    }

    fn read_slot(&self, page: &Page, slot_id: usize) -> Slot {
        let offset = Self::slot_offset(slot_id);
        let bytes = &page.data()[offset..offset + 8];
        Slot::deserialize(bytes.try_into().unwrap())
    }

    pub fn from_page(page: &Page) -> Self {
        let header_bytes = &page.data()[0..HEADER_SIZE];

        let page_header = PageHeader::deserialize(header_bytes.try_into().unwrap());

        let num_slots = page_header.num_slots as usize;

        let mut slots = Vec::with_capacity(num_slots);

        for slot_id in 0..num_slots {
            let offset = Self::slot_offset(slot_id);
            let bytes = &page.data()[offset..offset + SLOT_SIZE];

            let slot = Slot::deserialize(bytes.try_into().unwrap());

            slots.push(slot);
        }

        Self { page_header, slots }
    }

    fn add_slot(&mut self, offset: u32, length: u32) -> usize {
        let slot_id = self.slots.len();

        let slot = Slot { offset, length };

        self.slots.push(slot);

        self.page_header.num_slots += 1;
        self.page_header.free_space_start += SLOT_SIZE as u32;

        slot_id
    }

    pub fn insert_record(
        &mut self,
        page: &mut Page,
        record: &[u8],
    ) -> Result<usize, PageLayoutError> {
        let record_len = record.len();
        let free_start = self.page_header.free_space_start as usize;
        let free_end = self.page_header.free_space_end as usize;
        let required_space = record_len + SLOT_SIZE;

        if required_space > PAGE_SIZE - HEADER_SIZE {
            return Err(PageLayoutError::RecordTooLarge);
        }

        if required_space > free_end - free_start {
            return Err(PageLayoutError::InsufficientSpace);
        }

        let offset = free_end - record.len();

        page.data_mut()[offset..free_end].copy_from_slice(record);

        let slot_id = self.add_slot(offset as u32, record_len as u32);

        self.write_slot_to_page(page, slot_id);

        self.page_header.free_space_end = offset as u32;

        self.write_header_to_page(page);

        Ok(slot_id)
    }

    pub fn get_record(&self, page: &Page, slot_id: usize) -> Vec<u8> {
        let slot = self.read_slot(page, slot_id);
        let offset = slot.offset as usize;
        let lenght = slot.length as usize;

        let data = &page.data()[offset..offset + lenght];

        data.to_vec()
    }
}
