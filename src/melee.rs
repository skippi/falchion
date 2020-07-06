use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::{io, time};
use sysinfo::SystemExt;

use crate::dolphin::{DolphinHandle, LogicalAddress, ReadDolphinMemory};

pub trait Poll<T> {
    fn poll(&self) -> io::Result<T>;
}

pub struct Melee(DolphinHandle);

impl Melee {
    pub fn connect() -> io::Result<Melee> {
        let mut system = sysinfo::System::new();
        system.refresh_processes();
        DolphinHandle::locate(&system).map(|handle| Melee(handle))
    }
}

impl Poll<GameInfo> for Melee {
    fn poll(&self) -> io::Result<GameInfo> {
        let result = GameInfo {
            stage: self.poll()?,
            time: self.poll()?,
            status: self.poll()?,
        };
        Ok(result)
    }
}

impl Poll<Time> for Melee {
    fn poll(&self) -> io::Result<Time> {
        self.memread(LogicalAddress(0x8046B6C8), 4)
            .and_then(|bytes| bytes.as_slice().read_u32::<BigEndian>())
            .map(|seconds| time::Duration::from_secs(seconds.into()))
            .map(Time::from)
    }
}

impl Poll<StageId> for Melee {
    fn poll(&self) -> io::Result<StageId> {
        self.memread(LogicalAddress(0x8043208B), 1)
            .map(|bytes| StageId(bytes[0]))
    }
}

impl Poll<Status> for Melee {
    fn poll(&self) -> io::Result<Status> {
        // 0x8136F674 is the offset of first menu item in heap memory
        self.memread(LogicalAddress(0x8136F674), 1)
            .map(|bytes| match bytes[0] {
                0x81 => Status::InMenu, // 0x81 likely means heap alloc
                _ => Status::InGame,
            })
    }
}

impl ReadDolphinMemory for Melee {
    fn memread(&self, address: LogicalAddress, size: usize) -> io::Result<Vec<u8>> {
        self.0.memread(address, size)
    }
}

#[derive(Clone, Debug)]
pub struct GameInfo {
    pub time: Time,
    pub stage: StageId,
    pub status: Status,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct StageId(pub u8);

#[derive(Clone, Debug)]
pub enum Status {
    InMenu,
    InGame,
}

#[derive(Clone, Debug)]
pub struct Time(time::Duration);

impl From<time::Duration> for Time {
    fn from(duration: time::Duration) -> Self {
        Time(duration)
    }
}
