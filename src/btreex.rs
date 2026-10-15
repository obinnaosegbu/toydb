use crate::page::PageId;

const NODE_HEADER_SIZE: usize = 8;
const KEY_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Internal,
    Leaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordId {
    page_id: PageId,
    slot_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafNode {
    node_header: NodeHeader,
    keys: Vec<u64>,
    record_ids: Vec<RecordId>,
    next_leaf: Option<PageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalNode {
    node_header: NodeHeader,
    keys: Vec<u64>,
    children: Vec<PageId>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BTreeNode {
    Internal(InternalNode),
    Leaf(LeafNode)
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeHeader {
    node_type: NodeType,
    num_keys: u16,
    reserved: [u8; 5],
}

impl NodeHeader {
    fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            num_keys: 0,
            reserved: [0; 5],
        }
    }

    fn serialize(&self) -> [u8; NODE_HEADER_SIZE] {
        let mut bytes = [0u8; NODE_HEADER_SIZE];

        bytes[0] = self.node_type.to_bytes();
        bytes[1..3].copy_from_slice(&self.num_keys.to_le_bytes());
        bytes[3..8].copy_from_slice(&self.reserved);

        bytes
    }

    fn deserialize(bytes: [u8; NODE_HEADER_SIZE]) -> Option<Self> {
        let node_type = NodeType::from_bytes(bytes[0])?;

        let num_keys =
            u16::from_le_bytes(bytes[1..3].try_into().ok()?);

        let mut reserved = [0u8; 5];
        reserved.copy_from_slice(&bytes[3..8]);

        Some(Self {
            node_type,
            num_keys,
            reserved,
        })
    }
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
}

impl BTreeNode {
    pub fn node_type(&self) -> NodeType {
        match self {
            BTreeNode::Internal(_) => NodeType::Internal,
            BTreeNode::Leaf(_) => NodeType::Leaf,
        }
    }
}

impl NodeType {
    pub fn to_bytes(&self) -> u8 {
        match self {
            NodeType::Internal => 0,
            NodeType::Leaf => 1,
        }
    }

    pub fn from_bytes(value: u8) -> Option<Self> {
        if value == 0 {
            return  Some(NodeType::Internal);
        } else if value == 1 {
            return Some(NodeType::Leaf);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_header_round_trip() {
        let header = NodeHeader {
            node_type: NodeType::Internal,
            num_keys: 42,
            reserved: [0; 5],
        };

        let bytes = header.serialize();
        assert_eq!(bytes.len(), 8);

        let restored = NodeHeader::deserialize(bytes).unwrap();

        assert_eq!(restored, header);
    }

    #[test]
    fn invalid_node_type_is_rejected() {
        let mut bytes = [0u8; NODE_HEADER_SIZE];
        bytes[0] = 99;

        assert!(NodeHeader::deserialize(bytes).is_none());
    }
}