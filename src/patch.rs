use crate::handle::{Handle, HandleSnapshotFlag};
use crate::memory::{Memory, PageProtectionFlags};
use crate::pattern::Pattern;

use std::io::{ErrorKind, Read, Write};

/// memory section available for pattern matching
pub enum MemorySection<'a> {
    /// all memory section that patchable
    All,
    /// memory section that hold module binary like exe or dll
    Module(&'a str),
}

/// differnt way to get the base address for patching
pub enum BaseAddress<'a, const N: usize> {
    /// patch directly to the given address
    Direct(usize),
    /// verify value of the address before patching
    DirectVerify(usize, Pattern<N>),
    /// find value in the memory section that matches the pattern to get the address
    Pattern(Pattern<N>, MemorySection<'a>),
}

/// Patching a process
pub struct PatchHandle<'a> {
    handle: &'a Handle,
}

impl<'a> PatchHandle<'a> {
    /// create new instance for patching memory of the handle
    pub fn new(handle: &'a Handle) -> Self {
        Self { handle }
    }

    /// applying patches based on given config
    pub fn apply<'b, const N: usize, const M: usize, const K: usize>(
        &self,
        base_address: BaseAddress<N>,
        offsets: Option<&[usize; K]>,
        value: &[u8; M],
    ) -> Result<&Self, ErrorKind> {
        let mut addr: usize = match base_address {
            BaseAddress::Direct(addr) => Ok(addr),
            BaseAddress::DirectVerify(addr, pattern) => {
                let mut data = vec![0u8; N];
                let _ = self.read(addr, &mut data);
                if pattern == data.as_slice() {
                    Ok(addr)
                } else {
                    Err(ErrorKind::NotFound)
                }
            }
            BaseAddress::Pattern(pattern, mem_section) => {
                let step = 4;

                let address_ranges: Vec<(usize, usize)> = match mem_section {
                    MemorySection::All => {
                        let mut address_ranges = Vec::new();
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

                            address_ranges.push((mbi.get_base_address(), mbi.get_region_size()))
                        }
                        address_ranges
                    }
                    MemorySection::Module(module_name) => {
                        let module = self
                            .handle
                            .create_snapshot(
                                HandleSnapshotFlag::SnapModule | HandleSnapshotFlag::SnapModule32,
                            )?
                            .get_modules()
                            .find(|e| e.get_name() == *module_name)
                            .ok_or(ErrorKind::NotFound)?;

                        vec![(module.get_address(), module.get_size() as usize)]
                    }
                };

                let mut addr: Option<usize> = None;
                for address_range in address_ranges {
                    let mut data = vec![0u8; address_range.1 as usize];
                    let n = self.read(address_range.0, &mut data)?;
                    let data = &data[0..n];

                    match data
                        .windows(N)
                        .step_by(step)
                        .position(|e| e == pattern)
                        .map(|addr| address_range.0 + addr * step)
                    {
                        Some(v) => {
                            addr = Some(v);
                            break;
                        }
                        None => continue,
                    }
                }

                addr.ok_or(ErrorKind::NotFound)
            }
        }?;

        match offsets {
            Some(offsets) => {
                let mut data = Vec::from(0usize.to_ne_bytes());

                for offset in offsets {
                    let _ = self.read(addr, &mut data);
                    addr = usize::from_ne_bytes(*data.as_slice().first_chunk().unwrap()) + offset;
                }
            }
            None => (),
        }

        let mut memory = Memory::new(self.handle, addr, addr + M);
        let _ = memory.write(value).map_err(|e| e.kind())?;

        Ok(self)
    }

    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, ErrorKind> {
        let mut memory = Memory::new(self.handle, addr, usize::MAX);
        let n = memory.read(buf).map_err(|e| e.kind())?;
        Ok(n)
    }
}
