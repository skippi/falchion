use std::convert::TryInto;
use std::io;
use std::io::{Error, ErrorKind};
use read_process_memory::{Pid, TryIntoProcessHandle, copy_address};

const EMU_BASE_ADDRESS: usize = 0x7FFF0000;

fn read_stage(pid: Pid) -> io::Result<u8> {
    let handle = pid.try_into_process_handle()?;
    let offset = 0x8043208Busize - 0x80000000usize;
    let bytes = copy_address(EMU_BASE_ADDRESS + offset, 10, &handle)?;
    println!("{:X?}", bytes);

    let array: [u8; 1] = bytes[0..1].try_into().map_err(|e| Error::new(ErrorKind::Other, e))?;

    Ok(u8::from_le_bytes(array))
}

fn main() {
    let stage = read_stage(9552 as Pid).unwrap();
    println!("{}", stage);
    println!("Hello, world!");
}
