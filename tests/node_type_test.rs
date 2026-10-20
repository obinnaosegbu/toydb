use toydb::btree::node_type::NodeType;

#[test]
fn node_type_converts_to_bytes() {
    assert_eq!(NodeType::Internal.to_bytes(), 0);
    assert_eq!(NodeType::Leaf.to_bytes(), 1);
}

#[test]
fn node_type_converts_from_bytes() {
    assert_eq!(NodeType::from_bytes(0), Some(NodeType::Internal));
    assert_eq!(NodeType::from_bytes(1), Some(NodeType::Leaf));
    assert_eq!(NodeType::from_bytes(2), None);
}
