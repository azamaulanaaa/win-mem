use crate::handle::{Handle, HandleSnapshotFlag};
use crate::memory::{Memory, PageProtectionFlags};
use crate::pattern::Pattern;

use std::io::{ErrorKind, Read, Write};

/// Patching a process
pub struct PatchHandle<'a> {
    handle: &'a Handle,
}

impl<'a> PatchHandle<'a> {
    pub fn new(handle: &'a Handle) -> Self {
        Self { handle }
    }

    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, ErrorKind> {
        let mut memory = Memory::new(self.handle, addr, usize::MAX);
        let n = memory.read(buf).map_err(|e| e.kind())?;
        Ok(n)
    }

    /// direct patching to an address without checking
    pub fn direct<const M: usize>(&self, addr: usize, value: &[u8; M]) -> Result<&Self, ErrorKind> {
        let mut memory = Memory::new(self.handle, addr, addr + M);
        let _ = memory.write(value).map_err(|e| e.kind())?;
        Ok(self)
    }

    /// direct patching but verity current value before patching
    pub fn direct_verify<const N: usize, const M: usize>(
        &self,
        addr: usize,
        pattern: Pattern<N>,
        value: &[u8; M],
    ) -> Result<&Self, ErrorKind> {
        let mut data = vec![0u8; N];
        let _ = self.read(addr, &mut data);
        if pattern == data.as_slice() {
            self.direct(addr, value)?;
        } else {
            return Err(ErrorKind::NotFound);
        }

        Ok(self)
    }

    /// patching to a first match address that have value matchs the pattern
    pub fn pattern_matching<'b, const N: usize, const M: usize>(
        &self,
        module_name: Option<&'b str>,
        pattern: Pattern<N>,
        offset: usize,
        value: &[u8; M],
        step: usize,
    ) -> Result<&Self, ErrorKind> {
        match &module_name {
            Some(module_name) => {
                let module = self
                    .handle
                    .create_snapshot(
                        HandleSnapshotFlag::SnapModule | HandleSnapshotFlag::SnapModule32,
                    )?
                    .get_modules()
                    .find(|e| e.get_name() == *module_name)
                    .ok_or(ErrorKind::NotFound)?;

                let mut data = vec![0u8; module.get_size() as usize];
                let n = self.read(module.get_address(), &mut data)?;
                let data = &data[0..n];

                let addr = data
                    .windows(N)
                    .step_by(step)
                    .position(|e| e == pattern)
                    .map(|addr| addr * step + offset)
                    .ok_or(ErrorKind::NotFound)?;

                self.direct(module.get_address() + addr, value)?;
            }
            None => {
                for mbi in self.handle.get_memory_basic_informations() {
                    // NOTE: This might not correct filter to memory page
                    if mbi.get_protect().intersects(
                        PageProtectionFlags::NoAccess
                            | PageProtectionFlags::TargetsInvalid
                            | PageProtectionFlags::Guard,
                    ) || mbi.get_protect().is_empty()
                    {
                        continue;
                    }

                    let mut data = vec![0u8; mbi.get_region_size()];
                    let n = self.read(mbi.get_base_address(), &mut data)?;
                    let data = &data[0..n];

                    let addr = data
                        .windows(N)
                        .step_by(step)
                        .position(|e| e == pattern)
                        .map(|addr| addr * step + offset)
                        .ok_or(ErrorKind::NotFound);

                    match addr {
                        Ok(addr) => {
                            self.direct(mbi.get_base_address() + addr, value)?;
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }
        }

        Ok(self)
    }
}
