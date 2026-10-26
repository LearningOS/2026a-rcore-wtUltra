// Implementation assistance: Codex AI; disclosed in reports.
// Specification: LearningOS/rCore-Tutorial-Guide source/chapter8/5exercise.rst.
//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

mod deadlock;
pub use deadlock::DeadlockDetector;
