use crate::{
    btree::{node_header::NodeHeader, node_type::NodeType},
    page::{PAGE_SIZE, PageId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalNode {
    pub(super) node_header: NodeHeader,
    pub(super) keys: Vec<u64>,
    pub(super) children: Vec<PageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalNodeError {
    DuplicateKey(u64),
    FirstChildAlreadySet,
    MissingFirstChild,
    Full,
    NotFull,
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

    pub fn capacity(&self) -> usize {
        (PAGE_SIZE - 16) / 16
    }

    pub fn min_capacity(&self) -> usize {
        self.capacity() / 2
    }

    pub fn is_full(&self) -> bool {
        self.num_keys() >= self.capacity()
    }

    pub fn set_first_child(&mut self, page_id: PageId) -> Result<(), InternalNodeError> {
        if !self.children.is_empty() {
            return Err(InternalNodeError::FirstChildAlreadySet);
        }

        self.children.push(page_id);
        Ok(())
    }

    pub fn insert_separator(
        &mut self,
        seperator: u64,
        right_child: PageId,
    ) -> Result<(), InternalNodeError> {
        if self.is_full() {
            return Err(InternalNodeError::Full);
        }

        if self.children.is_empty() {
            return Err(InternalNodeError::MissingFirstChild);
        }

        let index = self.keys.partition_point(|&key| key < seperator);

        if index < self.keys.len() && self.keys[index] == seperator {
            return Err(InternalNodeError::DuplicateKey(seperator));
        }

        self.keys.insert(index, seperator);
        self.children.insert(index + 1, right_child);

        self.node_header.set_num_keys(self.keys.len());

        Ok(())
    }

    pub fn child_for_key(&self, key: u64) -> Option<PageId> {
        if self.children.is_empty() {
            return None;
        }

        let index = self.keys.partition_point(|&k| k <= key);

        self.children.get(index).copied()
    }

    pub fn insert_separator_and_split(
        &mut self,
        seperator: u64,
        right_child: PageId,
    ) -> Result<(u64, InternalNode), InternalNodeError> {
        if !self.is_full() {
            return Err(InternalNodeError::NotFull);
        }

        if self.children.is_empty() {
            return Err(InternalNodeError::MissingFirstChild);
        }

        let mut keys = self.keys.clone();
        let mut children = self.children.clone();

        let index = keys.partition_point(|&key| key < seperator);

        if index < keys.len() && keys[index] == seperator {
            return Err(InternalNodeError::DuplicateKey(seperator));
        }

        keys.insert(index, seperator);
        children.insert(index + 1, right_child);

        let midpoint = keys.len() / 2;
        let promoted_key = keys.remove(midpoint);

        let right_keys = keys.split_off(midpoint);
        let right_children = children.split_off(midpoint + 1);

        self.keys = keys;
        self.children = children;

        self.node_header.set_num_keys(self.keys.len());

        let mut right = InternalNode::new();

        right.keys = right_keys;
        right.children = right_children;

        right.node_header.set_num_keys(right.keys.len());

        Ok((promoted_key, right))
    }
}
