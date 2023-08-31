use std::cell::RefCell;

use ic_exports::stable_structures::memory_manager::MemoryId;
use ic_exports::stable_structures::DefaultMemoryImpl;
use crate::{Memory, MemoryManager};

mod btreemap;
mod cell;
mod log;
mod multimap;
mod unbounded;
mod vec;

pub use btreemap::StableBTreeMap;
pub use cell::StableCell;
pub use log::StableLog;
pub use multimap::StableMultimap;
pub use unbounded::StableUnboundedMap;
pub use vec::StableVec;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MANAGER: RefCell<MemoryManager> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// Return memory by `MemoryId`.
// Each instance of stable structures must have unique `MemoryId`;
pub fn get_memory_by_id(id: MemoryId) -> Memory {
    MANAGER.with(|mng| mng.borrow_mut().get(id))
}
