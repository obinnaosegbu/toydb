use crate::{
    btree::{
        node_header::NodeHeader, 
        node_type::NodeType, 
        record_id::RecordId
    }, 
    page::PageId
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafNode {
    pub(super) node_header: NodeHeader,
    pub(super) keys: Vec<u64>,
    pub(super) record_ids: Vec<RecordId>,
    pub(super) next_leaf: Option<PageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafNodeError {
    DuplicateKey(u64)
}
impl LeafNode {
    pub fn new() -> Self {
        let node_header = NodeHeader {
            node_type: NodeType::Leaf,
            num_keys: 0,
            reserved: [0u8; 5],
        };

        Self {
            node_header,
            keys: Vec::new(),
            record_ids: Vec::new(),
            next_leaf: None
        }
    }

    pub fn num_keys(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn keys(&self) -> &[u64] {
        &self.keys
    }

    pub fn next_leaf(&self) -> Option<PageId> {
        self.next_leaf
    }

    pub fn set_next_leaf(&mut self, next_leaf: Option<PageId>) {
        self.next_leaf = next_leaf;
    }

    pub fn find_key(&self, key: u64) -> Option<usize> {
        let mut left = 0;
        let mut right = self.keys.len();

        while left < right {
            let mid = left + (right - left) / 2;

            if self.keys[mid] == key {
                return Some(mid);
            } else if self.keys[mid] < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        return None;
    }
    
    pub fn get_record_id(&self, key: u64) -> Option<RecordId> {
        let pos = self.find_key(key)?;
        Some(self.record_ids[pos])
    }

    pub fn insert(&mut self, key: u64, record_id: RecordId) -> Result<(), LeafNodeError>{
        let mut left = 0;
        let mut right = self.keys.len();

        while left < right {
            let mid = left + (right - left) / 2;

            if self.keys[mid] < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        
        if left < self.keys.len() && self.keys[left] == key {
            return Err(LeafNodeError::DuplicateKey(key));
        }

        self.keys.insert(left, key);
        self.record_ids.insert(left, record_id);
        self.node_header.set_num_keys(self.keys.len());

        Ok(())
    }

    pub fn remove(&mut self, key: u64) -> bool {
        match self.find_key(key) {
            Some(pos) => {
                self.keys.remove(pos);
                self.record_ids.remove(pos);
                self.node_header.set_num_keys(self.keys.len());
                return true;
            }

            None => return false
        }
    }
}