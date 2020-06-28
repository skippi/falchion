mod game;

use game::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufReader;
use std::path::Path;

const MUSIC: &str = "C:/src/github.com/skippi/falchion/SnowDrop.mp3";

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
    fn play(&self, device: &rodio::Device) -> game::Result<()> {
        match self {
            Song::Local(path) => {
                let file = File::open(path).map_err(|_| Error::SongNotFound)?;
                rodio::play_once(&device, BufReader::new(file))
                    .unwrap()
                    .sleep_until_end()
            }
        }

        Ok(())
    }
}

fn main() -> game::Result<()> {
    let config = Config::open_or_create("config.json").unwrap_or(Config::spoof());

    if let Err(e) = config.save("config.json") {
        panic!(e);
    }

    let game = game::Game::locate()?;
    let stage = game.stage()?;
    let songs = config
        .playlists
        .get(&stage)
        .map(|s| s.as_slice())
        .unwrap_or(&[]);

    println!("{:?}", game.stage()?);

    let device = rodio::default_output_device().unwrap();
    let song = songs.iter().next().ok_or(Error::SongNotFound)?;
    song.play(&device)
}
