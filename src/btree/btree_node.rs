use crate::btree::{internal_node::InternalNode, leaf_node::LeafNode, node_type::NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BTreeNode {
    Internal(InternalNode),
    Leaf(LeafNode),
}

impl BTreeNode {
    pub fn node_type(&self) -> NodeType {
        match self {
            BTreeNode::Internal(_) => NodeType::Internal,
            BTreeNode::Leaf(_) => NodeType::Leaf,
        }
    }
}
