use std::ops::Deref;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W;

/// Look at [MODULEENTRY32W structure (tlhelp32.h) - Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/ns-tlhelp32-moduleentry32w)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Module(MODULEENTRY32W);

impl Module {
    /// get `th32ModuleID`
    pub fn get_module_id(&self) -> u32 {
        self.0.th32ModuleID
    }

    /// get `th32ProcessID`
    pub fn get_process_id(&self) -> u32 {
        self.0.th32ProcessID
    }

    /// get `modBaseAddr`
    pub fn get_address(&self) -> usize {
        self.0.modBaseAddr as usize
    }

    /// get `modBaseSize`
    pub fn get_size(&self) -> u32 {
        self.0.modBaseSize
    }

    /// get `hModule`
    pub fn get_hmodule(&self) -> HMODULE {
        self.0.hModule
    }

    /// get `szModule`
    pub fn get_name(&self) -> String {
        String::from_utf16_lossy(&self.0.szModule)
            .trim_end_matches("\u{0}")
            .to_string()
    }

    /// get `szExePath`
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
