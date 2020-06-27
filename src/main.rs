mod game;

use game::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

const MUSIC: &str = "C:/src/github.com/skippi/falchion/SnowDrop.mp3";

struct Config {
    playlists: HashMap<game::StageId, Vec<Song>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            playlists: HashMap::new(),
        }
    }
}

#[derive(Debug)]
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
    let mut config = Config::default();
    config
        .playlists
        .insert(game::StageId(8), vec![Song::Local(MUSIC.to_string())]);

    let game = game::Game::locate()?;
    let stage = game.stage()?;
    let songs = config.playlists.entry(stage).or_insert(vec![]);

    println!("{:?}", game.stage()?);

    let device = rodio::default_output_device().unwrap();
    let song = songs.iter().next().ok_or(Error::SongNotFound)?;
    song.play(&device)
}
