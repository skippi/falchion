use read_process_memory::TryIntoProcessHandle;
use std::io;
use sysinfo::{ProcessExt, SystemExt};

const LOGICAL_BASE_ADDRESS: LogicalAddress = LogicalAddress(0x80000000);
const PHYSICAL_BASE_ADDRESS: PhysicalAddress = PhysicalAddress(0x7FFF0000);

pub trait Poll<T> {
    fn poll(&self) -> io::Result<T>;
}

pub trait ReadDolphinMemory {
    fn memread(&self, address: LogicalAddress, size: usize) -> io::Result<Vec<u8>>;
}

#[derive(Clone)]
pub struct DolphinHandle(read_process_memory::ProcessHandle);

impl DolphinHandle {
    pub fn locate(system: &sysinfo::System) -> io::Result<DolphinHandle> {
        let pid = find_dolphin_pid(system)?;
        let handle = (pid as read_process_memory::Pid).try_into_process_handle()?;
        Ok(DolphinHandle(handle))
    }
}

impl ReadDolphinMemory for DolphinHandle {
    fn memread(&self, address: LogicalAddress, size: usize) -> io::Result<Vec<u8>> {
        read_process_memory::copy_address(PhysicalAddress::from(address).0, size, &self.0)
            .map_err(|e| io::Error::new(e.kind(), "could not read dolphin memory"))
    }
}

pub struct LogicalAddress(pub usize);

struct PhysicalAddress(usize);

impl From<LogicalAddress> for PhysicalAddress {
    fn from(addr: LogicalAddress) -> Self {
        PhysicalAddress(addr.0 - LOGICAL_BASE_ADDRESS.0 + PHYSICAL_BASE_ADDRESS.0)
    }
}

fn find_dolphin_pid(system: &sysinfo::System) -> io::Result<sysinfo::Pid> {
    system
        .get_processes()
        .iter()
        .find(|&(_, proc)| proc.name() == "Dolphin.exe")
        .map(|(pid, _)| *pid)
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "dolphin not found"))
}
