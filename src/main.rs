mod game;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufReader;
use std::path::Path;

const MUSIC: &str = "C:/src/github.com/skippi/falchion/SnowDrop.mp3";

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    AudioDecodeFailed,
    OutputDeviceNotFound,
    SongNotFound,
}

#[derive(Serialize, Deserialize)]
struct Config {
    playlists: HashMap<game::StageId, Vec<Song>>,
}

impl Config {
    fn open_or_create<P: AsRef<Path>>(path: P) -> io::Result<Config> {
        let config_file = OpenOptions::new().write(true).create(true).open(&path)?;

        serde_json::from_reader(config_file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).open(&path)?;

        serde_json::to_writer(&file, &self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn spoof() -> Config {
        let mut default_config = Config::default();
        default_config
            .playlists
            .insert(game::StageId(8), vec![Song::Local(MUSIC.to_string())]);

        default_config
    }

    fn pick_song(&self, stage: game::StageId) -> &Song {
        let songs = self
            .playlists
            .get(&stage)
            .map(|s| s.as_slice())
            .unwrap_or(&[]);

        songs.iter().next().unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            playlists: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Song {
    Local(String),
}

impl Song {
    fn play(&self, device: &rodio::Device) -> Result<rodio::Sink> {
        match self {
            Song::Local(path) => {
                let file = File::open(path).map_err(|_| Error::SongNotFound)?;
                rodio::play_once(&device, BufReader::new(file))
                    .map_err(|_| Error::AudioDecodeFailed)
            }
        }
    }
}

enum Event {
    GameStart(game::GameInfo),
    GameEnd,
}

struct App {
    config: Config,
    state: State,
}

enum State {
    WaitForGame,
    InGame(rodio::Sink),
}

impl App {
    fn new() -> Self {
        App {
            config: Config::open_or_create("config.json").unwrap_or(Config::spoof()),
            state: State::WaitForGame,
        }
    }

    fn next(self, event: Event) -> Result<Self> {
        let app = match (&self.state, event) {
            (State::WaitForGame, Event::GameStart(match_info)) => {
                let device = rodio::default_output_device().ok_or(Error::OutputDeviceNotFound)?;
                let song = self.config.pick_song(match_info.stage);
                let sink = song.play(&device)?;

                App {
                    state: State::InGame(sink),
                    ..self
                }
            }
            (State::InGame(sink), Event::GameEnd) => {
                sink.stop();

                App {
                    state: State::WaitForGame,
                    ..self
                }
            }
            _ => self,
        };

        Ok(app)
    }
}

fn main() -> game::Result<()> {
    let mut app = App::new();

    if let Err(e) = app.config.save("config.json") {
        panic!(e);
    }

    let melee = game::Melee::locate()?;
    let event = Event::GameStart(melee.game_info.clone());

    app = match app.next(event) {
        Ok(app) => app,
        Err(reason) => {
            println!("{:?}", reason);
            panic!();
        }
    };

    println!("{:?}", melee);

    loop {}
}
