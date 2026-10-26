//! Run with rustc +nightly-2024-05-02 --edition=2021 --test tests/deadlock.rs.
extern crate alloc;
#[path = "../os/src/sync/deadlock.rs"]
mod deadlock;
