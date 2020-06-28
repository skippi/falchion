use byteorder::{BigEndian, ReadBytesExt};
use read_process_memory::{copy_address, TryIntoProcessHandle};
use serde::{Deserialize, Serialize};
use std::time;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

const LOGICAL_BASE_ADDRESS: LogicalAddress = LogicalAddress(0x80000000);
const PHYSICAL_BASE_ADDRESS: PhysicalAddress = PhysicalAddress(0x7FFF0000);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DolphinAccessDenied,
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct StageId(pub u8);

pub struct Dolphin {
    pid: Pid,
}

impl Dolphin {
    pub fn locate() -> Result<Dolphin> {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = system
            .get_processes()
            .iter()
            .find(|&(_, proc)| proc.name() == "Dolphin.exe")
            .map(|(pid, _)| *pid)
            .ok_or(Error::DolphinNotFound)?;

        Ok(Dolphin { pid })
    }

    pub fn poll_match_info(&self) -> Result<MatchInfo> {
        let handle = (self.pid as read_process_memory::Pid)
            .try_into_process_handle()
            .map_err(|_| Error::DolphinAccessDenied)?;

        let stage = copy_address(
            PhysicalAddress::from(LogicalAddress(0x8043208B)).0,
            1,
            &handle,
        )
        .map(|bytes| StageId(bytes[0]))
        .map_err(|_| Error::InvalidMemoryRead)?;

        let time = copy_address(
            PhysicalAddress::from(LogicalAddress(0x8046B6C8)).0,
            4,
            &handle,
        )
        .and_then(|bytes| {
            println!("{:?}", &bytes);
            bytes.as_slice().read_u32::<BigEndian>()
        })
        .map(|seconds| time::Duration::from_secs(seconds.into()))
        .map_err(|_| Error::InvalidMemoryRead)?;

        let first_menu_item_byte = copy_address(
            PhysicalAddress::from(LogicalAddress(0x8136F674)).0,
            1,
            &handle,
        )
        .map(|bytes| bytes[0])
        .map_err(|_| Error::InvalidMemoryRead)?; // Complete hack. 0x8136F674 is the first menu item.

        let result = MatchInfo {
            stage,
            time,
            status: match first_menu_item_byte {
                0x81 => Status::InMenu, // Another hack. If addr starts with 0x81, then menu probably
                _ => Status::InGame,
            },
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct MatchInfo {
    pub time: time::Duration,
    pub stage: StageId,
    pub status: Status,
}

#[derive(Debug)]
pub enum Status {
    InMenu,
    InGame,
}
