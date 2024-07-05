use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};

use crate::handle::Handle;

pub struct Memory {
    handle: Handle,
    current_address: usize,
    start_address: usize,
    end_address: usize,
}

impl Memory {
    pub fn new(handle: Handle, start_address: usize, end_address: usize) -> Self {
        Self {
            handle,
            current_address: start_address,
            start_address,
            end_address,
        }
    }
}

impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n = 0usize;

        let _ = unsafe {
            ReadProcessMemory(
                *self.handle,
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

impl Write for Memory {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut n = 0usize;
        let _ = unsafe {
            WriteProcessMemory(
                *self.handle,
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

impl Seek for Memory {
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
