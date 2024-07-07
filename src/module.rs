use std::ops::Deref;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W;

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
