use crate::{
    btree::{
        btree_node::BTreeNode, node_header::NodeHeader, node_type::NodeType, record_id::RecordId,
        serialization,
    },
    page::{self, PAGE_SIZE, Page, PageId},
};

use super::{internal_node::InternalNode, leaf_node::LeafNode, node_header::NODE_HEADER_SIZE};

const LEAF_METADATA_SIZE: usize = 17;
const LEAF_ENTRY_SIZE: usize = 20;
const MAX_LEAF_KEYS: usize = (PAGE_SIZE - LEAF_METADATA_SIZE) / LEAF_ENTRY_SIZE;

const INTERNAL_METADATA_SIZE: usize = 8;
const INTERNAL_ENTRY_SIZE: usize = 16;
const MAX_INTERNAL_KEYS: usize = (PAGE_SIZE - INTERNAL_METADATA_SIZE - 8) / INTERNAL_ENTRY_SIZE;

#[derive(Debug, PartialEq, Eq)]
pub enum SerializationError {
    TooManyKeys,
    InvalidNodeType,
    InvalidNextLeafFlag,
    InvalidData,
}

pub fn serialize_leaf(node: &LeafNode) -> Result<[u8; PAGE_SIZE], SerializationError> {
    let mut bytes = [0u8; PAGE_SIZE];
    let num_keys = node.num_keys();

    if node.node_header.node_type != NodeType::Leaf {
        return Err(SerializationError::InvalidNodeType);
    }

    if num_keys > MAX_LEAF_KEYS {
        return Err(SerializationError::TooManyKeys);
    }

    if num_keys != node.record_ids.len()
        || node.node_header.num_keys as usize != num_keys
        || node.keys.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(SerializationError::InvalidData);
    }

    let header_bytes = node.node_header.serialize();
    bytes[..NODE_HEADER_SIZE].copy_from_slice(&header_bytes);

    match node.next_leaf {
        Some(page_id) => {
            bytes[NODE_HEADER_SIZE] = 1;
            bytes[NODE_HEADER_SIZE + 1..LEAF_METADATA_SIZE]
                .copy_from_slice(&page_id.0.to_le_bytes());
        }
        None => {
            bytes[NODE_HEADER_SIZE] = 0;
        }
    }

    let mut offset = LEAF_METADATA_SIZE;

    for (key, record_id) in node.keys.iter().zip(&node.record_ids) {
        bytes[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
        offset += 8;

        bytes[offset..offset + 8].copy_from_slice(&record_id.page_id().0.to_le_bytes());
        offset += 8;

        bytes[offset..offset + 4].copy_from_slice(&record_id.slot_id().to_le_bytes());
        offset += 4;
    }

    Ok(bytes)
}

pub fn deserialize_leaf(bytes: [u8; PAGE_SIZE]) -> Result<LeafNode, SerializationError> {
    let header_bytes: [u8; NODE_HEADER_SIZE] = bytes[..NODE_HEADER_SIZE]
        .try_into()
        .map_err(|_| SerializationError::InvalidData)?;

    let node_header =
        NodeHeader::deserialize(header_bytes).ok_or(SerializationError::InvalidNodeType)?;

    if node_header.node_type != NodeType::Leaf {
        return Err(SerializationError::InvalidNodeType);
    }

    let num_keys = node_header.num_keys as usize;

    if num_keys > MAX_LEAF_KEYS {
        return Err(SerializationError::TooManyKeys);
    }

    let flag = bytes[NODE_HEADER_SIZE];

    if flag != 0 && flag != 1 {
        return Err(SerializationError::InvalidNextLeafFlag);
    }

    let next_leaf = match flag {
        1 => {
            let page_id = u64::from_le_bytes(
                bytes[NODE_HEADER_SIZE + 1..LEAF_METADATA_SIZE]
                    .try_into()
                    .map_err(|_| SerializationError::InvalidData)?,
            );

            Some(PageId(page_id))
        }
        0 => None,
        _ => return Err(SerializationError::InvalidNextLeafFlag),
    };

    let mut keys: Vec<u64> = Vec::with_capacity(num_keys);
    let mut record_ids: Vec<RecordId> = Vec::with_capacity(num_keys);

    let mut offset = LEAF_METADATA_SIZE;

    for _ in 0..num_keys {
        let key = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| SerializationError::InvalidData)?,
        );
        offset += 8;

        let page_id = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| SerializationError::InvalidData)?,
        );
        offset += 8;

        let slot_id = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| SerializationError::InvalidData)?,
        );
        offset += 4;

        if keys.last().is_some_and(|&previous| previous >= key) {
            return Err(SerializationError::InvalidData);
        }

        keys.push(key);
        record_ids.push(RecordId::new(PageId(page_id), slot_id));
    }

    Ok(LeafNode {
        node_header,
        keys,
        record_ids,
        next_leaf,
    })
}

pub fn serialize_internal(node: &InternalNode) -> Result<[u8; PAGE_SIZE], SerializationError> {
    let mut bytes = [0u8; PAGE_SIZE];
    let num_keys = node.num_keys();

    if node.node_header.node_type != NodeType::Internal {
        return Err(SerializationError::InvalidNodeType);
    }

    if num_keys > MAX_INTERNAL_KEYS {
        return Err(SerializationError::TooManyKeys);
    }

    if node.children.len() != num_keys + 1
        || node.node_header.num_keys as usize != num_keys
        || node.keys.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(SerializationError::InvalidData);
    }

    let header_bytes = node.node_header.serialize();
    bytes[..NODE_HEADER_SIZE].copy_from_slice(&header_bytes);

    let mut offset = INTERNAL_METADATA_SIZE;

    for key in &node.keys {
        bytes[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
        offset += 8;
    }

    for child in &node.children {
        bytes[offset..offset + 8].copy_from_slice(&child.0.to_le_bytes());
        offset += 8;
    }

    Ok(bytes)
}

pub fn deserialize_internal(bytes: [u8; PAGE_SIZE]) -> Result<InternalNode, SerializationError> {
    let header_bytes: [u8; NODE_HEADER_SIZE] = bytes[..NODE_HEADER_SIZE]
        .try_into()
        .map_err(|_| SerializationError::InvalidData)?;

    let node_header =
        NodeHeader::deserialize(header_bytes).ok_or(SerializationError::InvalidNodeType)?;

    if node_header.node_type != NodeType::Internal {
        return Err(SerializationError::InvalidNodeType);
    }

    let num_keys = node_header.num_keys as usize;

    if num_keys > MAX_INTERNAL_KEYS {
        return Err(SerializationError::TooManyKeys);
    }

    let mut keys: Vec<u64> = Vec::with_capacity(num_keys);
    let mut children: Vec<PageId> = Vec::with_capacity(num_keys + 1);

    let mut offset = INTERNAL_METADATA_SIZE;

    for _ in 0..num_keys {
        let key = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| SerializationError::InvalidData)?,
        );
        offset += 8;

        if keys.last().is_some_and(|&previous| previous >= key) {
            return Err(SerializationError::InvalidData);
        }

        keys.push(key);
    }

    for _ in 0..num_keys + 1 {
        let page_id = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| SerializationError::InvalidData)?,
        );
        offset += 8;

        children.push(PageId(page_id));
    }

    Ok(InternalNode {
        node_header,
        keys,
        children,
    })
}

pub fn node_to_page(page_id: PageId, node: &BTreeNode) -> Result<Page, SerializationError> {
    let serialization = match node {
        BTreeNode::Leaf(leaf) => serialize_leaf(leaf)?,
        BTreeNode::Internal(internal) => serialize_internal(internal)?,
    };

    let mut page = Page::new(page_id);

    page.data_mut().copy_from_slice(&serialization);

    Ok(page)
}

pub fn page_to_node(page: &Page) -> Result<BTreeNode, SerializationError> {
    let bytes = page.data();

    match NodeType::from_bytes(bytes[0]) {
        Some(NodeType::Leaf) => {
            let node = deserialize_leaf(*bytes)?;
            Ok(BTreeNode::Leaf(node))
        }

        Some(NodeType::Internal) => {
            let node = deserialize_internal(*bytes)?;
            Ok(BTreeNode::Internal(node))
        }

        None => Err(SerializationError::InvalidNodeType),
    }
}
