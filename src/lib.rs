#![warn(missing_docs)]

//! # Examples
//!
//!
//! ## Plant Vs Zombie (GOTY)
//!
//! example as dll injection paylod patching Plant Vs Zombie (GOTY) 32bit to never lost suns.
//!
//! ```rust
//! use win_mem::{handle::Handle, patch::{BaseAddress, MemorySection, PatchHandle}, pattern::Pattern};
//! use windows::Win32::Foundation::{BOOL, HANDLE};
//!
//! #[no_mangle]
//! #[allow(non_snake_case, unused_variables)]
//! extern "system" fn DllMain(dll_module: HANDLE, call_reason: u32, lpv_reserved: &u32) -> BOOL {
//!    return match call_reason {
//!        1 => on_process_attach(),
//!        _ => BOOL(0),
//!    };
//! }
//!
//! fn on_process_attach() -> BOOL {
//!    let handle = Handle::default();
//!    let patch_handle = PatchHandle::new(&handle);
//!
//!    let _ = patch_handle.apply(
//!        BaseAddress::Pattern(
//!            Pattern::from([Some(0x2B), Some(0xF3), Some(0x89), Some(0xB7)]),
//!            MemorySection::Module("PlantsVsZombies.exe"),
//!        ),
//!        None::<&[usize; 0]>,
//!        &[0x90, 0x90],
//!    );
//!
//!    return BOOL(0);
//! }
//! ```

/// relating to the process of a process.
pub mod handle;
/// relating to physical memory and virtual memory.
pub mod memory;
/// relating to bytes that loaded by a process.
pub mod module;
/// relating to helper to patch memory.
pub mod patch;
/// simple matching hopefuly fast for bytes.
pub mod pattern;
