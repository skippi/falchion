use serde::{Deserialize, Serialize};
use std::io;
use sysinfo::SystemExt;

use crate::dolphin::{DolphinHandle, LogicalAddress, Poll, ReadDolphinMemory};

#[derive(Clone)]
pub struct Melee(pub DolphinHandle);

impl Melee {
    pub fn connect() -> io::Result<Melee> {
        let mut system = sysinfo::System::new();
        system.refresh_processes();
        DolphinHandle::locate(&system).map(|handle| Melee(handle))
    }
}

impl Poll<GameInfo> for Melee {
    fn poll(&self) -> io::Result<GameInfo> {
        let info = GameInfo {
            stage: self.poll()?,
            status: self.poll()?,
        };
        Ok(info)
    }
}

impl Poll<StageId> for Melee {
    fn poll(&self) -> io::Result<StageId> {
        self.memread(LogicalAddress(0x8045AC67), 1)
            .map(|bytes| StageId(bytes[0]))
    }
}

impl Poll<Status> for Melee {
    fn poll(&self) -> io::Result<Status> {
        let pause_byte = self.memread(LogicalAddress(0x80479D68), 1)?[0];
        let ingame_byte = self.memread(LogicalAddress(0x80479D88), 1)?[0];
        let status = match (ingame_byte, pause_byte) {
            (_, 0x10) => Status::Menu, // 0x02 paused, 0x10 game ended
            (_, 0x02) => Status::Paused,
            (0, _) => Status::Menu, // 0 in menu, some address otherwise
            _ => Status::Playing,
        };
        Ok(status)
    }
}

impl ReadDolphinMemory for Melee {
    fn memread(&self, address: LogicalAddress, size: usize) -> io::Result<Vec<u8>> {
        self.0.memread(address, size)
    }
}

#[derive(Clone, Debug)]
pub struct GameInfo {
    pub stage: StageId,
    pub status: Status,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct StageId(pub u8);

#[derive(Clone, Debug)]
pub enum Status {
    Menu,
    Playing,
    Paused,
}
