use crate::melee::{GameInfo, Status};

pub trait Advance<T> {
    type Output;
    fn advance(self, tag: T) -> Self::Output;
}

pub trait Detect<T> {
    fn detect(&self, new: &Self) -> Vec<T>;
}

impl Detect<Event> for GameInfo {
    fn detect(&self, new: &Self) -> Vec<Event> {
        use Event::*;
        use Status::*;
        match (&self.status, &new.status) {
            (InGame, InMenu) => vec![GameLeave(GameLeaveEvent)],
            (InMenu, InGame) => vec![GameJoin(GameJoinEvent { info: new.clone() })],
            _ => vec![],
        }
    }
}

pub trait TryAdvance<T> {
    type Error;
    type Output;
    fn try_advance(self, tag: T) -> Result<Self::Output, Self::Error>;
}

pub enum Event {
    GameJoin(GameJoinEvent),
    GameLeave(GameLeaveEvent),
}

pub struct GameLeaveEvent;

pub struct GameJoinEvent {
    pub info: GameInfo,
}
