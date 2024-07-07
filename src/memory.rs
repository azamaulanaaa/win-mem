use bitflags::bitflags;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    MEMORY_BASIC_INFORMATION, PAGE_PROTECTION_FLAGS, PAGE_TYPE, VIRTUAL_ALLOCATION_TYPE,
};

use crate::handle::Handle;

/// Wrapper for memory that act like io
pub struct Memory<'a> {
    handle: &'a Handle,
    current_address: usize,
    start_address: usize,
    end_address: usize,
}

impl<'a> Memory<'a> {
    pub fn new(handle: &'a Handle, start_address: usize, end_address: usize) -> Self {
        Self {
            handle,
            current_address: start_address,
            start_address,
            end_address,
        }
    }
}

impl<'a> Read for Memory<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n = 0usize;

        let _ = unsafe {
            ReadProcessMemory(
                **self.handle,
                self.current_address as *const _,
                buf as *mut [u8] as *mut _,
                buf.len().min(self.end_address - self.current_address),
                Some(&mut n),
            )
        }
        .map_err(|_| ErrorKind::Other)?;

        self.current_address += n;

        if n < buf.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        return Ok(n);
    }
}

impl<'a> Write for Memory<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut n = 0usize;
        let _ = unsafe {
            WriteProcessMemory(
                **self.handle,
                self.current_address as *const _,
                buf as *const [u8] as *const _,
                buf.len(),
                Some(&mut n),
            )
        }
        .map_err(|_| ErrorKind::Other)?;

        self.current_address += n;

        if n < buf.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        return Ok(n);
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Seek for Memory<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        return match pos {
            SeekFrom::Start(value) => {
                self.current_address = usize::checked_add(
                    self.start_address,
                    usize::try_from(value).map_err(|_| ErrorKind::InvalidData)?,
                )
                .ok_or(ErrorKind::InvalidData)?;
                Ok(self.current_address as u64)
            }
            SeekFrom::Current(value) => {
                if value > 0 {
                    self.current_address = usize::checked_add(
                        self.current_address,
                        usize::try_from(value).map_err(|_| ErrorKind::InvalidData)?,
                    )
                    .ok_or(ErrorKind::InvalidData)?;
                    Ok(self.current_address as u64)
                } else {
                    self.current_address = usize::checked_sub(
                        self.current_address,
                        usize::try_from(-value).map_err(|_| ErrorKind::InvalidData)?,
                    )
                    .ok_or(ErrorKind::InvalidData)?;
                    Ok(self.current_address as u64)
                }
            }
            SeekFrom::End(value) => {
                self.current_address = usize::checked_sub(
                    self.end_address,
                    usize::try_from(-value).map_err(|_| ErrorKind::InvalidData)?,
                )
                .ok_or(ErrorKind::InvalidData)?;
                Ok(self.current_address as u64)
            }
        };
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PageProtectionFlags: u32 {
        const Execute = 0x10;
        const ExecuteRead = 0x20;
        const ExecuteReadWrite = 0x40;
        const ExecuteWriteCopy = 0x80;
        const NoAccess = 0x01;
        const ReadOnly = 0x02;
        const ReadWrite = 0x04;
        const WriteCopy = 0x08;
        const TargetsInvalid = 0x40000000;
        const TargetNoUpdate = 0x40000000;

        const Guard = 0x100;
        const NoCache = 0x200;
        const WriteCombine = 0x400;
    }
}

impl Into<PAGE_PROTECTION_FLAGS> for PageProtectionFlags {
    fn into(self) -> PAGE_PROTECTION_FLAGS {
        PAGE_PROTECTION_FLAGS(self.bits())
    }
}

impl TryFrom<PAGE_PROTECTION_FLAGS> for PageProtectionFlags {
    type Error = ErrorKind;

    fn try_from(value: PAGE_PROTECTION_FLAGS) -> Result<Self, Self::Error> {
        Self::from_bits(value.0).ok_or(ErrorKind::Unsupported)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VirtualAllocationType: u32 {
        const Commit = 0x1000;
        const Free = 0x10000;
        const Reserve = 0x2000;
    }
}

impl Into<VIRTUAL_ALLOCATION_TYPE> for VirtualAllocationType {
    fn into(self) -> VIRTUAL_ALLOCATION_TYPE {
        VIRTUAL_ALLOCATION_TYPE(self.bits())
    }
}

impl TryFrom<VIRTUAL_ALLOCATION_TYPE> for VirtualAllocationType {
    type Error = ErrorKind;

    fn try_from(value: VIRTUAL_ALLOCATION_TYPE) -> Result<Self, Self::Error> {
        Self::from_bits(value.0).ok_or(ErrorKind::Unsupported)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PageType: u32 {
        const Image = 0x1000000;
        const Mapped = 0x40000;
        const Private = 0x20000;
    }
}

impl Into<PAGE_TYPE> for PageType {
    fn into(self) -> PAGE_TYPE {
        PAGE_TYPE(self.bits())
    }
}

impl TryFrom<PAGE_TYPE> for PageType {
    type Error = ErrorKind;

    fn try_from(value: PAGE_TYPE) -> Result<Self, Self::Error> {
        Self::from_bits(value.0).ok_or(ErrorKind::Unsupported)
    }
}

/// Look at [MEMORY_BASIC_INFORMATION (winnt.h) Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-memory_basic_information)
pub struct MemoryBasicInformation(MEMORY_BASIC_INFORMATION);

impl MemoryBasicInformation {
    /// get `BaseAddress`
    pub fn get_base_address(&self) -> usize {
        self.0.BaseAddress as usize
    }

    /// get `AllocationBase`
    pub fn get_allocation_base(&self) -> usize {
        self.0.AllocationBase as usize
    }

    /// get `AllocationProtect`
    pub fn get_allocation_protect(&self) -> PageProtectionFlags {
        PageProtectionFlags::try_from(self.0.AllocationProtect).unwrap()
    }

    /// get `PatitionId`
    #[cfg(target_arch = "x86_64")]
    pub fn get_partition_id(&self) -> u16 {
        self.0.PartitionId
    }

    /// get `RegionSize`
    pub fn get_region_size(&self) -> usize {
        self.0.RegionSize
    }

    /// get `State`
    pub fn get_state(&self) -> VirtualAllocationType {
        VirtualAllocationType::try_from(self.0.State).unwrap()
    }

    /// get `Protect`
    pub fn get_protect(&self) -> PageProtectionFlags {
        PageProtectionFlags::try_from(self.0.Protect).unwrap()
    }

    /// get `Type`
    pub fn get_type(&self) -> PageType {
        PageType::try_from(self.0.Type).unwrap()
    }
}

impl From<MEMORY_BASIC_INFORMATION> for MemoryBasicInformation {
    fn from(value: MEMORY_BASIC_INFORMATION) -> Self {
        Self(value)
    }
}
