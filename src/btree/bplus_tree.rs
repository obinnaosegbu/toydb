use std::{collections::HashSet, iter::zip, path, range};

use crate::{
    btree::{
        btree_node::BTreeNode,
        internal_node::InternalNode,
        leaf_node::{LeafNode, LeafNodeError},
        node_storage::{NodeStorage, NodeStorageError},
        record_id::RecordId,
    },
    page::{Page, PageId},
};

pub struct BPlusTree {
    root_page_id: PageId,
    storage: NodeStorage,
}

pub enum BPlusTreeValidationError {
    Storage(NodeStorageError),
    UnsortedKeys,
    InvalidChildCount,
    RepeatedPage(PageId),
    InvalidKeyRange,
    LeafLinkMismatch,
    UnequalLeafDepth,
}

impl BPlusTree {
    pub fn new(mut storage: NodeStorage) -> Result<Self, NodeStorageError> {
        let leaf_node = LeafNode::new();
        let btree_node = BTreeNode::Leaf(leaf_node);

        let root_page_id = storage.allocate_node(&btree_node)?;

        Ok(Self {
            root_page_id,
            storage,
        })
    }

    pub fn root_page_id(&self) -> PageId {
        self.root_page_id
    }

    pub fn search(&mut self, key: u64) -> Result<Option<RecordId>, NodeStorageError> {
        let mut page_id = self.root_page_id;

        loop {
            let node = self.storage.read_node(page_id)?;

            match node {
                BTreeNode::Leaf(leaf_node) => {
                    return Ok(leaf_node.get_record_id(key));
                }

                BTreeNode::Internal(internal_node) => {
                    page_id = internal_node
                        .child_for_key(key)
                        .ok_or(NodeStorageError::MissingChildPointer)?;
                }
            }
        }
    }

    pub fn validate(&mut self) -> Result<(), BPlusTreeValidationError> {
        let mut visited = HashSet::new();
        let mut leaf_order = Vec::new();
        let mut leaf_depths = Vec::new();

        self.validate_subtree(
            self.root_page_id,
            &mut visited,
            &mut leaf_order,
            &mut leaf_depths,
            0,
        )?;
        self.validate_link_links(&leaf_order)?;

        if let Some(&firt_depth) = leaf_depths.first() {
            if leaf_depths.iter().any(|&depth| depth != firt_depth) {
                return Err(BPlusTreeValidationError::UnequalLeafDepth);
            }
        } else {
            return Err(BPlusTreeValidationError::UnequalLeafDepth);
        }

        Ok(())
    }

    pub fn validate_link_links(
        &mut self,
        leaf_order: &[PageId],
    ) -> Result<(), BPlusTreeValidationError> {
        for (index, &page_id) in leaf_order.iter().enumerate() {
            let expected_next_leaf = leaf_order.get(index + 1).copied();

            let node = self
                .storage
                .read_node(page_id)
                .map_err(BPlusTreeValidationError::Storage)?;

            match node {
                BTreeNode::Internal(_) => {
                    return Err(BPlusTreeValidationError::LeafLinkMismatch);
                }
                BTreeNode::Leaf(leaf_node) => {
                    if leaf_node.next_leaf() != expected_next_leaf {
                        return Err(BPlusTreeValidationError::LeafLinkMismatch);
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_subtree(
        &mut self,
        page_id: PageId,
        visited: &mut HashSet<PageId>,
        leaf_order: &mut Vec<PageId>,
        leaf_depths: &mut Vec<usize>,
        depth: usize,
    ) -> Result<Option<(u64, u64)>, BPlusTreeValidationError> {
        if !visited.insert(page_id) {
            return Err(BPlusTreeValidationError::RepeatedPage(page_id));
        }

        // 2. Read the current node.
        let node = self
            .storage
            .read_node(page_id)
            .map_err(BPlusTreeValidationError::Storage)?;

        match node {
            BTreeNode::Leaf(leaf) => {
                let keys = leaf.keys();

                if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
                    return Err(BPlusTreeValidationError::UnsortedKeys);
                }

                leaf_order.push(page_id);
                leaf_depths.push(depth);

                Ok(keys.first().copied().zip(keys.last().copied()))
            }

            BTreeNode::Internal(internal) => {
                let keys = internal.keys();
                let children = internal.children();

                // 4. Internal keys must be strictly increasing.
                if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
                    return Err(BPlusTreeValidationError::UnsortedKeys);
                }

                // 5. An internal node needs keys.len() + 1 children.
                if children.len() != keys.len() + 1 {
                    return Err(BPlusTreeValidationError::InvalidChildCount);
                }

                let mut ranges = Vec::with_capacity(children.len());

                for &child in children {
                    let range =
                        self.validate_subtree(child, visited, leaf_order, leaf_depths, depth + 1)?;
                    ranges.push(range);
                }

                for (index, &key) in keys.iter().enumerate() {
                    if let Some((_, left_max)) = ranges[index] {
                        if left_max >= key {
                            return Err(BPlusTreeValidationError::InvalidKeyRange);
                        }
                    }

                    if let Some((right_min, _)) = ranges[index + 1] {
                        if right_min < key {
                            return Err(BPlusTreeValidationError::InvalidKeyRange);
                        }
                    }
                }

                let min = ranges.iter().find_map(|range| range.map(|r| r.0));

                let max = ranges.iter().rev().find_map(|range| range.map(|r| r.1));

                return Ok(min.zip(max));
            }
        }
    }

    pub fn insert_into_parent(
        &mut self,
        mut ancestors: Vec<PageId>,
        mut left_page_id: PageId,
        mut seperator: u64,
        mut right_page_id: PageId,
    ) -> Result<(), NodeStorageError> {
        loop {
            let Some(parent_page_id) = ancestors.pop() else {
                let mut new_root = InternalNode::new();

                new_root
                    .set_first_child(left_page_id)
                    .map_err(NodeStorageError::InternalNodeError)?;

                new_root
                    .insert_separator(seperator, right_page_id)
                    .map_err(NodeStorageError::InternalNodeError)?;

                self.root_page_id = self.storage.allocate_node(&BTreeNode::Internal(new_root))?;

                return Ok(());
            };

            let mut parent_node = match self.storage.read_node(parent_page_id)? {
                BTreeNode::Internal(internal_node) => internal_node,
                BTreeNode::Leaf(_) => return Err(NodeStorageError::UnsupportedOperation),
            };

            if !parent_node.is_full() {
                parent_node
                    .insert_separator(seperator, right_page_id)
                    .map_err(NodeStorageError::InternalNodeError)?;

                self.storage
                    .write_node(parent_page_id, &BTreeNode::Internal(parent_node))?;

                return Ok(());
            }

            let (promoted_key, right_internal) = parent_node
                .insert_separator_and_split(seperator, right_page_id)
                .map_err(NodeStorageError::InternalNodeError)?;

            self.storage
                .write_node(parent_page_id, &BTreeNode::Internal(parent_node))?;

            let new_right_page_id = self
                .storage
                .allocate_node(&BTreeNode::Internal(right_internal))?;

            left_page_id = parent_page_id;
            seperator = promoted_key;
            right_page_id = new_right_page_id;
        }
    }

    pub fn insert(&mut self, key: u64, record_id: RecordId) -> Result<(), NodeStorageError> {
        let mut page_id = self.root_page_id;
        let mut ancestors: Vec<PageId> = Vec::new();

        loop {
            let node = self.storage.read_node(page_id)?;

            match node {
                BTreeNode::Internal(internal_node) => {
                    ancestors.push(page_id);

                    page_id = internal_node
                        .child_for_key(key)
                        .ok_or(NodeStorageError::MissingChildPointer)?;
                }

                BTreeNode::Leaf(mut leaf) => {
                    match leaf.insert(key, record_id) {
                        Ok(()) => {
                            let update_node = BTreeNode::Leaf(leaf);
                            self.storage.write_node(page_id, &update_node)?;

                            return Ok(());
                        }
                        Err(LeafNodeError::DuplicateKey(key)) => {
                            return Err(NodeStorageError::LeafNodeError(
                                LeafNodeError::DuplicateKey(key),
                            ));
                        }

                        Err(LeafNodeError::NotFull) => {
                            unreachable!("insert should not retrun NotFull")
                        }

                        Err(LeafNodeError::Full) => {}
                    }

                    let mut right = leaf.split().map_err(NodeStorageError::LeafNodeError)?;
                    let seperator = right.keys()[0];

                    if key < seperator {
                        leaf.insert(key, record_id)
                            .map_err(NodeStorageError::LeafNodeError)?;
                    } else {
                        right
                            .insert(key, record_id)
                            .map_err(NodeStorageError::LeafNodeError)?;
                    }

                    right.set_next_leaf(leaf.next_leaf());

                    let right_page_id = self
                        .storage
                        .allocate_node(&BTreeNode::Leaf(right.clone()))?;

                    leaf.set_next_leaf(Some(right_page_id));
                    self.storage.write_node(page_id, &BTreeNode::Leaf(leaf))?;

                    self.insert_into_parent(ancestors, page_id, seperator, right_page_id)?;

                    return Ok(());
                }
            }
        }
    }
}
