use std::fs;

use toydb::{
    btree::{
        bplus_tree::BPlusTree, internal_node::InternalNode, node_storage::NodeStorage,
        record_id::RecordId,
    },
    buffer_pool::BufferPoolManager,
    disk_manager::DiskManager,
    page::PageId,
};

fn create_test_tree() -> BPlusTree {
    let path: std::path::PathBuf =
        std::env::temp_dir().join(format!("toydb_{}.db", std::process::id()));

    let _ = fs::remove_file(&path);
    let path_string = path.to_string_lossy().into_owned();

    let disk_manager = DiskManager::open(&path_string).unwrap();
    let buffer_pool = BufferPoolManager::new(disk_manager, 1);
    let storage = NodeStorage::new(buffer_pool);
    let tree = BPlusTree::new(storage).unwrap();

    tree
}


#[test]
fn insert_and_search_keys() {
    let mut tree = create_test_tree();

    let records = [
        (10, RecordId::new(PageId(100), 0)),
        (20, RecordId::new(PageId(100), 1)),
        (30, RecordId::new(PageId(101), 0)),
    ];

    for (key, record_id) in records {
        tree.insert(key, record_id).unwrap();
    }

    for (key, expected_record_id) in records {
        let actual = tree.search(key).unwrap();
        assert_eq!(actual, Some(expected_record_id));
    }

    assert_eq!(tree.search(99).unwrap(), None);
}

#[test]
fn root_leaf_splits_and_search_still_works() {
    let mut tree = create_test_tree();

    // Fill the original leaf to its capacity of 203 keys.
    for key in 0..203u64 {
        let record_id = RecordId::new(PageId(100), key as u32);
        tree.insert(key, record_id).unwrap();
    }

    // The next insertion should trigger a root-leaf split.
    let record_id = RecordId::new(PageId(100), 203);
    tree.insert(203, record_id).unwrap();

    // Verify keys from both sides of the split.
    for key in [0, 50, 101, 150, 202, 203] {
        let result = tree.search(key).unwrap();
        assert_eq!(result, Some(RecordId::new(PageId(100), key as u32)));
    }

    // Verify a missing key is still reported correctly.
    assert_eq!(tree.search(999).unwrap(), None);
}

#[test]
fn reverse_insertion_split_and_duplicate_handling() {
    let mut tree = create_test_tree();

    // Insert 203 keys in reverse order.
    for key in (1..=203u64).rev() {
        tree.insert(key, RecordId::new(PageId(100), key as u32))
            .unwrap();
    }

    // This insertion should trigger the root-leaf split.
    tree.insert(0, RecordId::new(PageId(100), 0)).unwrap();

    // Every key should remain searchable.
    for key in 0..=203u64 {
        assert_eq!(
            tree.search(key).unwrap(),
            Some(RecordId::new(PageId(100), key as u32)),
            "Failed to find key {key}"
        );
    }

    // A duplicate must be rejected.
    assert!(tree.insert(100, RecordId::new(PageId(999), 999)).is_err());

    // The original value must remain unchanged.
    assert_eq!(
        tree.search(100).unwrap(),
        Some(RecordId::new(PageId(100), 100))
    );
}

#[test]
fn non_root_leaf_split_updates_parent() {
    let mut tree = create_test_tree();

    // Trigger the first root split.
    for key in 0..=203u64 {
        tree.insert(key, RecordId::new(PageId(100), key as u32))
            .unwrap();
    }

    // Continue inserting into the left leaf until it splits.
    // The original left leaf contains keys 0..101.
    for key in 204..407u64 {
        tree.insert(key, RecordId::new(PageId(100), key as u32))
            .unwrap();
    }

    // Verify that keys on both sides of the new split remain searchable.
    for key in 0..=203u64 {
        assert_eq!(
            tree.search(key).unwrap(),
            Some(RecordId::new(PageId(100), key as u32)),
            "Failed to find key {key}"
        );
    }
}

#[test]
fn insert_separator_and_split_preserves_node_invariants() {
    let mut node = InternalNode::new();

    // An internal node with 255 keys needs 256 children.
    node.set_first_child(PageId(0)).unwrap();

    for i in 0..255u64 {
        node.insert_separator(i * 2, PageId(i + 1)).unwrap();
    }

    assert_eq!(node.num_keys(), node.capacity());
    assert_eq!(node.children().len(), node.keys().len() + 1);

    // Insert an additional separator into the full node.
    let separator = 255u64;
    let right_child = PageId(999);

    let (promoted, right) = node
        .insert_separator_and_split(separator, right_child)
        .unwrap();

    // The promoted key must be the middle key of the
    // combined, sorted key sequence.
    assert_eq!(promoted, 255);

    // Both resulting nodes must preserve the invariant.
    assert_eq!(node.children().len(), node.keys().len() + 1);
    assert_eq!(right.children().len(), right.keys().len() + 1);

    assert!(node.num_keys() <= node.capacity());
    assert!(right.num_keys() <= right.capacity());

    // The left keys must all be smaller than the promoted key.
    assert!(node.keys().iter().all(|&key| key < promoted));

    // The right keys must all be greater than the promoted key.
    assert!(right.keys().iter().all(|&key| key > promoted));
}

#[test]
fn recursive_internal_splits_preserve_search() {
    let mut tree = create_test_tree();

    // Insert enough ascending keys to force internal-node overflow.
    for key in 0..30_000u64 {
        let record_id = RecordId::new(PageId(100), key as u32);
        tree.insert(key, record_id).unwrap();
    }

    // Verify that keys across the tree remain searchable.
    for key in 0..30_000u64 {
        let expected = RecordId::new(PageId(100), key as u32);

        assert_eq!(
            tree.search(key).unwrap(),
            Some(expected),
            "Failed to find key {key}"
        );
    }

    assert_eq!(tree.search(30_000).unwrap(), None);
}

#[test]
fn valid_tree_passes_validation() {
    let mut tree = create_test_tree();

    for key in 0..1_000_u64 {
        let record_id = RecordId::new(PageId(100), key as u32);
        tree.insert(key, record_id).unwrap();
    }

    assert!(tree.validate().is_ok());
}


#[test]
fn test_remove_existing_key() {
    let mut tree = create_test_tree();

    tree.insert(10, RecordId::new(PageId(1), 0)).unwrap();
    tree.insert(20, RecordId::new(PageId(2), 0)).unwrap();
    tree.insert(30, RecordId::new(PageId(3), 0)).unwrap();

    assert!(tree.remove(20).unwrap());
    assert_eq!(tree.search(20).unwrap(), None);

    assert!(tree.search(10).unwrap().is_some());
    assert!(tree.search(30).unwrap().is_some());
}

#[test]
fn test_remove_missing_key() {
    let mut tree = create_test_tree();

    tree.insert(10, RecordId::new(PageId(1), 0)).unwrap();

    assert!(!tree.remove(99).unwrap());
    assert!(tree.search(10).unwrap().is_some());
}

#[test]
fn test_remove_preserves_other_records() {
    let mut tree = create_test_tree();

    for key in 1..=20 {
        tree.insert(key, RecordId::new(PageId(key), 0)).unwrap();
    }

    assert!(tree.remove(10).unwrap());

    for key in 1..=20 {
        if key == 10 {
            assert_eq!(tree.search(key).unwrap(), None);
        } else {
            assert!(tree.search(key).unwrap().is_some());
        }
    }
}

#[test]
fn test_remove_from_multi_level_tree() {
    let mut tree = create_test_tree();

    for key in 1..=1000 {
        tree.insert(key, RecordId::new(PageId(key), 0)).unwrap();
    }

    for key in [1, 250, 500, 750, 1000] {
        assert!(tree.remove(key).unwrap());
        assert_eq!(tree.search(key).unwrap(), None);
    }

    for key in 1..=1000 {
        if ![1, 250, 500, 750, 1000].contains(&key) {
            assert!(tree.search(key).unwrap().is_some());
        }
    }
}
