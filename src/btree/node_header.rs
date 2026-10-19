use crate::btree::node_type::NodeType;

pub(super) const NODE_HEADER_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeHeader {
    pub node_type: NodeType,
    pub num_keys: u16,
    pub reserved: [u8; 5],
}

impl NodeHeader {
    fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            num_keys: 0,
            reserved: [0; 5],
        }
    }

    pub fn serialize(&self) -> [u8; NODE_HEADER_SIZE] {
        let mut bytes = [0u8; NODE_HEADER_SIZE];

        bytes[0] = self.node_type.to_bytes();
        bytes[1..3].copy_from_slice(&self.num_keys.to_le_bytes());
        bytes[3..8].copy_from_slice(&self.reserved);

        bytes
    }

    pub fn deserialize(bytes: [u8; NODE_HEADER_SIZE]) -> Option<Self> {
        let node_type = NodeType::from_bytes(bytes[0])?;

        let num_keys = u16::from_le_bytes(bytes[1..3].try_into().ok()?);

        let mut reserved = [0u8; 5];
        reserved.copy_from_slice(&bytes[3..8]);

        Some(Self {
            node_type,
            num_keys,
            reserved,
        })
    }

    pub fn set_num_keys(&mut self, count: usize) {
        self.num_keys = count as u16
    }
}
