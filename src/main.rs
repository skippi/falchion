mod game;

use game::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

const MUSIC: &str = "C:/src/github.com/skippi/falchion/SnowDrop.mp3";

#[derive(Debug)]
enum Song {
    LocalFile(String),
}

impl Song {
    fn play(&self, device: &rodio::Device) -> game::Result<()> {
        match self {
            Song::LocalFile(path) => {
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
    let mut config: HashMap<u8, Vec<Song>> = HashMap::new();
    config.insert(8, vec![Song::LocalFile(MUSIC.to_string())]);

    let game = game::Game::locate()?;
    let stage = game.stage()?;
    let songs = config.entry(stage).or_insert(vec![]);

    println!("{}", game.stage()?);

    let device = rodio::default_output_device().unwrap();
    let song = songs.iter().next().ok_or(Error::SongNotFound)?;
    song.play(&device)
}
