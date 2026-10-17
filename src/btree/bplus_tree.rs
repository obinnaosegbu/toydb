use crate::{
    btree::{
        btree_node::BTreeNode, internal_node::InternalNode, leaf_node::{LeafNode, LeafNodeError}, node_storage::{
            NodeStorage, 
            NodeStorageError
        }, record_id::RecordId
    }, page::PageId
};

pub struct BPlusTree {
    root_page_id: PageId,
    storage: NodeStorage
}

impl BPlusTree {
    pub fn new(mut storage: NodeStorage) -> Result<Self, NodeStorageError> {
        let leaf_node = LeafNode::new();
        let btree_node = BTreeNode::Leaf(leaf_node);

        let root_page_id = storage.allocate_node(&btree_node)?;

        Ok(
            Self {
                root_page_id,
                storage
            }
        )
    }

    pub fn root_page_id(&self) -> PageId {
        self.root_page_id
    }

    pub fn search(&mut self, key: u64) -> Result<Option<RecordId>, NodeStorageError>{
        let mut page_id = self.root_page_id;

        loop {
            let node = self.storage.read_node(page_id)?;
            
            match node {
                BTreeNode::Leaf(leaf_node) => {
                    return Ok(leaf_node.get_record_id(key));
                }

                BTreeNode::Internal(internal_node) => {
                    page_id = internal_node.child_for_key(key)
                        .ok_or(NodeStorageError::MissingChildPointer)?;
                }

            }
        }
    }

    pub fn insert(&mut self, key: u64, record_id: RecordId) -> Result<(), NodeStorageError>{
        let mut page_id = self.root_page_id;
        let mut parent_page_id: Option<PageId> = None;
        
        loop {
            let node = self.storage.read_node(page_id)?;

            match node {
                BTreeNode::Leaf(mut leaf) => {
                    match leaf.insert(key, record_id) {
                        Ok(()) => {
                            let update_node = BTreeNode::Leaf(leaf);
                            self.storage.write_node(page_id, &update_node)?;

                            return Ok(());
                        },
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

                    if page_id != self.root_page_id {
                        return Err(NodeStorageError::UnsupportedOperation);
                    }

                    let mut right = leaf.split()
                        .map_err(NodeStorageError::LeafNodeError)?;

                    let seperator = right.keys()[0];

                    if key < seperator {
                        leaf.insert(key, record_id)
                            .map_err(NodeStorageError::LeafNodeError);
                    } else {
                        right.insert(key, record_id)
                            .map_err(NodeStorageError::LeafNodeError);
                    }

                    let right_page_id = self.storage.allocate_node(
                        &BTreeNode::Leaf(right.clone()),
                    )?;

                    leaf.set_next_leaf(Some(right_page_id));

                    self.storage.write_node(page_id, &BTreeNode::Leaf(leaf))?;
                    self.storage.write_node(right_page_id, &BTreeNode::Leaf(right))?;
                    
                    let mut new_root = InternalNode::new();
                    new_root.set_first_child(page_id).map_err(NodeStorageError::InternalNodeError)?;
                    
                    new_root.insert_separator(seperator, right_page_id)
                        .map_err(NodeStorageError::InternalNodeError)?;

                    let new_root_page_id = self.storage.allocate_node(&BTreeNode::Internal(new_root))?;

                    self.root_page_id = new_root_page_id;
                    return Ok(());
                }

                BTreeNode::Internal(internal_node) => {
                    parent_page_id = Some(page_id);

                    page_id = internal_node.child_for_key(key)
                        .ok_or(NodeStorageError::MissingChildPointer)?;
                }

            }
        }
    }
}