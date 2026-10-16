use crate::{
    btree::{
        btree_node::BTreeNode, 
        leaf_node::LeafNode, 
        node_storage::{
            NodeStorage, 
            NodeStorageError
        }, 
        record_id::RecordId
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
        
        loop {
            let node = self.storage.read_node(page_id)?;

            match node {
                BTreeNode::Leaf(mut leaf_node) => {
                    leaf_node.insert(key, record_id)
                        .map_err(NodeStorageError::LeafNodeError)?;
                    
                    let update_node = BTreeNode::Leaf(leaf_node);
                    self.storage.write_node(page_id, &update_node)?;

                    return Ok(());
                }

                BTreeNode::Internal(internal_node) => {
                    page_id = internal_node.child_for_key(key)
                        .ok_or(NodeStorageError::MissingChildPointer)?;
                }

            }
        }
    }
}