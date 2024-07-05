use std::ffi::CStr;
use std::io::ErrorKind;
use std::ops::Deref;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W;

use crate::handle::Handle;
use crate::memory::Memory;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Module(MODULEENTRY32W);

impl Module {
    pub fn get_module_id(&self) -> u32 {
        self.0.th32ModuleID
    }

    pub fn get_process_id(&self) -> u32 {
        self.0.th32ProcessID
    }

    pub fn get_address(&self) -> usize {
        self.0.modBaseAddr as usize
    }

    pub fn get_size(&self) -> u32 {
        self.0.modBaseSize
    }

    pub fn get_hmodule(&self) -> HMODULE {
        self.0.hModule
    }

    pub fn get_name(&self) -> String {
        String::from_utf16_lossy(&self.0.szModule)
            .trim_end_matches("\u{0}")
            .to_string()
    }

    pub fn get_path(&self) -> String {
        String::from_utf16_lossy(&self.0.szExePath)
            .trim_end_matches("\u{0}")
            .to_string()
    }
}

impl Deref for Module {
    type Target = MODULEENTRY32W;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MODULEENTRY32W> for Module {
    fn from(value: MODULEENTRY32W) -> Self {
        Self(value)
    }
}

impl TryInto<Memory> for Module {
    type Error = ErrorKind;

    fn try_into(self) -> Result<Memory, Self::Error> {
        let handle = Handle::try_from(self.get_process_id())?;
        Ok(Memory::new(
            handle,
            self.get_address(),
            self.get_address() + self.get_size() as usize,
        ))
    }
}
