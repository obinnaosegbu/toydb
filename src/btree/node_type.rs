#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Internal,
    Leaf,
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