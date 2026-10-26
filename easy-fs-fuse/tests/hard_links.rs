//! Regression coverage authored with Codex assistance from the official lab6
//! hard-link specification (not a replacement for the official checker).
use easy_fs::{BlockDevice, EasyFileSystem};
use std::sync::{Arc, Mutex};

struct MemoryDevice(Mutex<Vec<[u8; 512]>>);
impl BlockDevice for MemoryDevice {
    fn read_block(&self, id: usize, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0.lock().unwrap()[id]);
    }
    fn write_block(&self, id: usize, bytes: &[u8]) {
        self.0.lock().unwrap()[id].copy_from_slice(bytes);
    }
}

#[test]
fn hard_link_identity_unlink_and_reuse() {
    let device = Arc::new(MemoryDevice(Mutex::new(vec![[0; 512]; 4096])));
    let fs = EasyFileSystem::create(device, 4096, 1);
    let root = EasyFileSystem::root_inode(&fs);
    assert!(root.create("").is_none());
    assert!(root.create("a/name").is_none());
    assert!(root.create(&"x".repeat(28)).is_none());
    let file = root.create("original").unwrap();
    let id = file.inode_id();
    assert_eq!(file.link_count(), 1);
    assert!(root.link("original", "alias"));
    assert!(!root.link("missing", "other"));
    assert!(!root.link("original", "original"));
    assert!(!root.link("original", "alias"));
    assert_eq!(file.link_count(), 2);
    let alias = root.find("alias").unwrap();
    assert_eq!(alias.inode_id(), id);
    file.write_at(0, b"shared inode");
    let mut bytes = [0; 12];
    assert_eq!(alias.read_at(0, &mut bytes), 12);
    assert_eq!(&bytes, b"shared inode");
    root.unlink("original").unwrap();
    assert!(root.find("original").is_none());
    assert_eq!(alias.link_count(), 1);
    root.unlink("alias").unwrap();
    assert_eq!(alias.link_count(), 0);
    // VFS unlink does not prematurely free an open file; the OS decides when
    // its final open description disappears, then invokes reclaim.
    assert_eq!(alias.read_at(0, &mut bytes), 12);
    assert!(root.find("").is_none());
    assert!(root.ls().is_empty());
    assert!(root.unlink("alias").is_none());
    drop(file);
    alias.reclaim();
    drop(alias);
    // Both the directory slot and inode id must remain reusable well beyond
    // the initial number of slots; large data writes exercise indirect blocks.
    for _ in 0..100 {
        let file = root.create("reused").unwrap();
        assert_eq!(file.inode_id(), id);
        let data = vec![0x5a; 80 * 512];
        assert_eq!(file.write_at(0, &data), data.len());
        root.unlink("reused").unwrap();
        assert!(root.ls().is_empty());
        file.reclaim();
    }
}
