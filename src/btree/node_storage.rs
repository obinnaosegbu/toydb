use crate::{
    btree::{
        btree_node::BTreeNode,
        internal_node::InternalNodeError,
        leaf_node::LeafNodeError,
        serialization::{SerializationError, node_to_page, page_to_node},
    },
    buffer_pool::BufferPoolManager,
    page::PageId,
};

#[derive(Debug)]
pub enum NodeStorageError {
    BufferPoolError,
    Serialization(SerializationError),
    MissingChildPointer,
    LeafNodeError(LeafNodeError),
    InternalNodeError(InternalNodeError),
    UnsupportedOperation,
}

pub struct NodeStorage {
    buffer_pool: BufferPoolManager,
}

impl NodeStorage {
    pub fn new(buffer_pool: BufferPoolManager) -> Self {
        Self { buffer_pool }
    }

    pub fn read_node(&mut self, page_id: PageId) -> Result<BTreeNode, NodeStorageError> {
        let page = self
            .buffer_pool
            .fetch_page(page_id)
            .ok_or(NodeStorageError::BufferPoolError)?;

        let result = page_to_node(&page).map_err(NodeStorageError::Serialization);
        self.buffer_pool.unpin_page(page_id, false);

        result
    }

    pub fn write_node(
        &mut self,
        page_id: PageId,
        node: &BTreeNode,
    ) -> Result<(), NodeStorageError> {
        let serialized_page =
            node_to_page(page_id, node).map_err(NodeStorageError::Serialization)?;

        let page = self
            .buffer_pool
            .fetch_page_mut(page_id)
            .ok_or(NodeStorageError::BufferPoolError)?;

        page.data_mut().copy_from_slice(serialized_page.data());
        self.buffer_pool.unpin_page(page_id, true);

        Ok(())
    }

    pub fn allocate_node(&mut self, node: &BTreeNode) -> Result<PageId, NodeStorageError> {
        let serialized_page =
            node_to_page(PageId(0), node).map_err(NodeStorageError::Serialization)?;

        let page_id = self
            .buffer_pool
            .new_page()
            .ok_or(NodeStorageError::BufferPoolError)?;

        let write_result = {
            let page = self.buffer_pool.fetch_page_mut(page_id);

            match page {
                Some(page) => {
                    page.data_mut().copy_from_slice(serialized_page.data());

                    true
                }
                None => false,
            }
        };

        if !write_result {
            self.buffer_pool.unpin_page(page_id, false);
            return Err(NodeStorageError::BufferPoolError);
        }

        self.buffer_pool.unpin_page(page_id, true);
        self.buffer_pool.unpin_page(page_id, true);

        Ok(page_id)
    }
}
