mod data;
mod dolphin;
mod event;
mod melee;

use std::time::Duration;
use std::{io, thread};

use crate::data::Config;
use crate::event::{
    Advance, Detect, Event, GameJoinEvent, GameLeaveEvent, GamePauseEvent, GameResumeEvent,
    TryAdvance,
};
use crate::melee::{GameInfo, Melee, Poll};

enum App {
    Waiting(Waiting),
    Playing(Playing),
}

impl TryAdvance<Event> for App {
    type Error = io::Error;
    type Output = App;

    fn try_advance(self, tag: Event) -> Result<Self::Output, Self::Error> {
        use App::*;
        use Event::*;
        let new_app = match (self, tag) {
            (Waiting(state), GameJoin(event)) => Playing(state.try_advance(event)?),
            (Playing(state), GameLeave(event)) => Waiting(state.advance(event)),
            (Playing(state), GamePause(event)) => Playing(state.advance(event)),
            (Playing(state), GameResume(event)) => Playing(state.advance(event)),
            (app, _) => app,
        };
        Ok(new_app)
    }
}

struct Waiting;

impl TryAdvance<GameJoinEvent> for Waiting {
    type Error = io::Error;
    type Output = Playing;

    fn try_advance(self, tag: GameJoinEvent) -> Result<Self::Output, Self::Error> {
        let config = Config::fetch("config.json");
        let song = config
            .pick_song(tag.info.stage)
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "no song found"))?;
        let device = rodio::default_output_device().unwrap();
        let sink = song.play(&device)?;
        sink.set_volume(0.5);
        Ok(Playing(sink))
    }
}

struct Playing(rodio::Sink);

impl Advance<GameLeaveEvent> for Playing {
    type Output = Waiting;

    fn advance(self, _: GameLeaveEvent) -> Self::Output {
        self.0.stop();
        Waiting
    }
}

impl Advance<GamePauseEvent> for Playing {
    type Output = Playing;

    fn advance(self, _: GamePauseEvent) -> Self::Output {
        self.0.set_volume(0.5 * 0.2); // 0x804D388B volume address
        self
    }
}

impl Advance<GameResumeEvent> for Playing {
    type Output = Playing;

    fn advance(self, _: GameResumeEvent) -> Self::Output {
        self.0.set_volume(0.5);
        self
    }
}

fn main() {
    loop {
        if let Err(e) = step() {
            eprintln!("error: {}", e);
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn step() -> io::Result<()> {
    let melee = Melee::connect()?;
    let mut game: GameInfo = melee.poll()?;
    let mut app = App::Waiting(Waiting);
    loop {
        let new_game: GameInfo = melee.poll()?;
        let events: Vec<Event> = game.detect(&new_game);
        app = events.into_iter().try_fold(app, App::try_advance)?;
        game = new_game;
        thread::sleep(Duration::from_millis(5));
    }
}
