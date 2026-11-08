#![allow(unused_imports)]

pub use std::sync::*;

pub use portable_atomic as atomic;
pub use portable_atomic_util::{Arc, task::Wake};

pub use parking_lot::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
    RwLockUpgradableReadGuard,
    RecursiveRwLockWriteGuard,
    MappedRwLockReadGuard,
    MappedRwLockWriteGuard,
    Mutex,
    MutexGuard,
};
