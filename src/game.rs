use read_process_memory::{copy_address, TryIntoProcessHandle};
use std::convert::TryInto;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

const LOGICAL_BASE_ADDRESS: LogicalAddress = LogicalAddress(0x80000000);
const PHYSICAL_BASE_ADDRESS: PhysicalAddress = PhysicalAddress(0x7FFF0000);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DolphinNotFound,
    InvalidMemoryRead,
    SongNotFound,
}

struct LogicalAddress(usize);

struct PhysicalAddress(usize);

impl From<LogicalAddress> for PhysicalAddress {
    fn from(addr: LogicalAddress) -> Self {
        PhysicalAddress(addr.0 - LOGICAL_BASE_ADDRESS.0 + PHYSICAL_BASE_ADDRESS.0)
    }
}

pub struct Game {
    pid: Pid,
}

impl Game {
    pub fn locate() -> Result<Game> {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = system
            .get_processes()
            .iter()
            .find(|&(_, proc)| proc.name() == "Dolphin.exe")
            .map(|(pid, _)| *pid)
            .ok_or(Error::DolphinNotFound)?;

        Ok(Game { pid })
    }

    pub fn stage(&self) -> Result<u8> {
        self.read_byte(LogicalAddress(0x8043208B))
    }

    fn read_byte(&self, addr: LogicalAddress) -> Result<u8> {
        let handle = (self.pid as read_process_memory::Pid)
            .try_into_process_handle()
            .map_err(|_| Error::DolphinNotFound)?;
        let physical_address = PhysicalAddress::from(addr);
        let bytes =
            copy_address(physical_address.0, 1, &handle).map_err(|_| Error::InvalidMemoryRead)?;

        let array: [u8; 1] = bytes.as_slice().try_into().unwrap();

        Ok(u8::from_le_bytes(array))
    }
}
