use crate::{btree::{node_header::NodeHeader, node_type::NodeType}, page::PageId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalNode {
    pub(super) node_header: NodeHeader,
    pub(super) keys: Vec<u64>,
    pub(super) children: Vec<PageId>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalNodeError {
    DuplicateKey(u64),
    FirstChildAlreadySet,
    MissingFirstChild,
}

impl InternalNode {
    pub fn new() -> Self {
        let node_header = NodeHeader {
            node_type: NodeType::Internal,
            num_keys: 0,
            reserved: [0u8; 5],
        };

        Self {
            node_header,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn num_keys(&self) -> usize {
        self.keys.len()
    }

    pub fn keys(&self) -> &[u64] {
        &self.keys
    }

    pub fn children(&self) -> &[PageId] {
        &self.children
    }

    pub fn set_first_child(&mut self, page_id: PageId) -> Result<(), InternalNodeError> {
        if !self.children.is_empty() {
            return Err(InternalNodeError::FirstChildAlreadySet);
        }

        self.children.push(page_id);
        Ok(())
    }

    pub fn insert_separator(&mut self, key: u64, right_child: PageId) -> Result<(), InternalNodeError>{
        if self.children.is_empty(){
            return Err(InternalNodeError::MissingFirstChild);
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
            return Err(InternalNodeError::DuplicateKey(key));
        }

        self.keys.insert(left, key);
        self.children.insert(left + 1, right_child);
        self.node_header.set_num_keys(self.keys.len());

        Ok(())
    }

    pub fn child_for_key(&self, key: u64) -> Option<PageId> {
        let num_keys = self.keys.len();

        if self.children.is_empty() {
            return None;
        }

        let mut left = 0;
        let mut right = num_keys;

        while left < right {
            let mid = left + (right - left) / 2;

            if self.keys[mid] > key {
                right = mid;
            } else {
                left = mid + 1
            }
        }

        self.children.get(left).copied()
    }

}