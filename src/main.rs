use read_process_memory::{copy_address, Pid, TryIntoProcessHandle};
use std::convert::TryInto;
use std::io;
use std::io::{Error, ErrorKind};

const LOGICAL_BASE_ADDRESS: LogicalAddress = LogicalAddress(0x80000000);
const PHYSICAL_BASE_ADDRESS: PhysicalAddress = PhysicalAddress(0x7FFF0000);

struct LogicalAddress(usize);

struct PhysicalAddress(usize);

impl From<LogicalAddress> for PhysicalAddress {
    fn from(addr: LogicalAddress) -> Self {
        PhysicalAddress(addr.0 - LOGICAL_BASE_ADDRESS.0 + PHYSICAL_BASE_ADDRESS.0)
    }
}

fn read_byte(pid: Pid, address: LogicalAddress) -> io::Result<u8> {
    let handle = pid.try_into_process_handle()?;
    let physical_address = PhysicalAddress::from(address);
    let bytes = copy_address(physical_address.0, 1, &handle)?;

    let array: [u8; 1] = bytes[0..1]
        .try_into()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    Ok(u8::from_le_bytes(array))
}

fn read_stage(pid: Pid) -> io::Result<u8> {
    read_byte(pid, LogicalAddress(0x8043208B))
}

fn main() {
    let stage = read_stage(9552 as Pid).unwrap();
    println!("{}", stage);
    println!("Hello, world!");
}
