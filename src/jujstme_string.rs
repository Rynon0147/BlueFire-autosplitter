use crate::{string::ArrayWString, Address64, Error, Process};
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CSharpString {
    address: Address64,
}

impl CSharpString {
    // Returns the string's content
    pub fn read<const N: usize>(&self, process: &Process) -> Result<ArrayWString<N>, Error> {
        process.read(self.address + 0x14)
    }

    #[cfg(feature = "alloc")]
    pub fn read_as_string(&self, process: &Process) -> Result<String, Error> {
        let length = self.get_length(process)?.min(255);

        let mut buf = Vec::<u16>::with_capacity(length);
        let uninit = buf.spare_capacity_mut();
        process.read_into_uninit_slice(self.address + 0x14, uninit)?;

        // SAFETY:
        // - len() is equal to the capacity of the Vec
        // - The elements of the buffer are initialized by the previous read_into_uninit_slice function
        unsafe {
            buf.set_len(length);
        }

        match String::from_utf16(&buf) {
            Err(_) => Err(Error {}),
            Ok(x) => Ok(x),
        }
    }

    /// Retrieves the actual length of the current string
    pub fn get_length(&self, process: &Process) -> Result<usize, Error> {
        match process.read::<u32>(self.address + 0x10) {
            Ok(x) => Ok(x as _),
            _ => Err(Error {}),
        }
    }
}