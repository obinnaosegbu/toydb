use std::fs;

use toydb::{
    btree::{
        btree_node::BTreeNode, leaf_node::LeafNode, node_storage::NodeStorage, record_id::RecordId,
    },
    buffer_pool::BufferPoolManager,
    disk_manager::DiskManager,
    page::PageId,
};

#[test]
fn leaf_node_survives_eviction_and_reload() {
    let path = std::env::temp_dir().join(format!("toydb_test_{}.db", std::process::id()));

    let _ = fs::remove_file(&path);
    let path_string = path.to_string_lossy().into_owned();

    let disk_manager = DiskManager::open(&path_string).unwrap();
    let buffer_pool = BufferPoolManager::new(disk_manager, 1);
    let mut storage = NodeStorage::new(buffer_pool);

    // Create the original leaf node.
    let mut leaf = LeafNode::new();

    leaf.insert(10, RecordId::new(PageId(100), 0)).unwrap();
    leaf.insert(20, RecordId::new(PageId(100), 1)).unwrap();
    leaf.insert(30, RecordId::new(PageId(101), 0)).unwrap();

    let original = BTreeNode::Leaf(leaf);

    // Store the first node.
    let first_page_id = storage.allocate_node(&original).unwrap();

    // Allocating another node forces eviction with a one-frame pool.
    let second_node = BTreeNode::Leaf(LeafNode::new());
    storage.allocate_node(&second_node).unwrap();

    // The first node must now be fetched back from disk.
    let recovered = storage.read_node(first_page_id).unwrap();

    assert_eq!(recovered, original);

    drop(storage);
    fs::remove_file(&path).unwrap();
}

#[test]
fn internal_node_survives_eviction_and_reload() {
    use toydb::btree::internal_node::InternalNode;

    let path = std::env::temp_dir().join(format!("toydb_internal_test_{}.db", std::process::id()));

    let _ = std::fs::remove_file(&path);
    let path_string = path.to_string_lossy().into_owned();

    let disk_manager = DiskManager::open(&path_string).unwrap();
    let buffer_pool = BufferPoolManager::new(disk_manager, 1);
    let mut storage = NodeStorage::new(buffer_pool);

    // Create an internal node with two separator keys.
    let mut internal = InternalNode::new();

    let _ = internal.set_first_child(PageId(10));
    internal.insert_separator(20, PageId(11)).unwrap();
    internal.insert_separator(40, PageId(12)).unwrap();

    let original = BTreeNode::Internal(internal);

    // Store the internal node.
    let first_page_id = storage.allocate_node(&original).unwrap();

    // Force the first node out of the one-frame buffer pool.
    let second_node = BTreeNode::Leaf(LeafNode::new());
    storage.allocate_node(&second_node).unwrap();

    // Reload and reconstruct the original internal node.
    let recovered = storage.read_node(first_page_id).unwrap();

    assert_eq!(recovered, original);

    drop(storage);
    std::fs::remove_file(&path).unwrap();
}
