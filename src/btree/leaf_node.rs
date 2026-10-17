use crate::{
    btree::{
        node_header::NodeHeader, 
        node_type::NodeType, 
        record_id::RecordId
    }, page::{PAGE_SIZE, PageId}
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafNode {
    pub(super) node_header: NodeHeader,
    pub(super) keys: Vec<u64>,
    pub(super) record_ids: Vec<RecordId>,
    pub(super) next_leaf: Option<PageId>,
}

const LEAF_METADATA_SIZE: usize = 17;
const LEAF_ENTRY_SIZE: usize = 20;
const LEAF_CAPACITY: usize = (PAGE_SIZE - LEAF_METADATA_SIZE) / LEAF_ENTRY_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafNodeError {
    DuplicateKey(u64),
    Full,
    NotFull,
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

    pub fn is_full(&self) -> bool {
        self.keys.len()>= LEAF_CAPACITY
    }

    pub fn capacity(&self) -> usize {
        LEAF_CAPACITY
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

        if self.is_full() {
            return Err(LeafNodeError::Full);
        }

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

    pub fn split(&mut self) -> Result<LeafNode, LeafNodeError>{
        if !self.is_full(){
            return Err(LeafNodeError::NotFull);
        }

        let midpoint = self.num_keys() / 2;

        let mut right = LeafNode::new();

        right.keys = self.keys.split_off(midpoint);
        right.record_ids = self.record_ids.split_off(midpoint);

        right.next_leaf = self.next_leaf;
        self.next_leaf = None;

        self.node_header.set_num_keys(self.keys.len());
        right.node_header.set_num_keys(right.keys.len());

        Ok(right)
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