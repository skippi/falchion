use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use crate::melee::StageId;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub playlists: HashMap<StageId, Vec<Song>>,
}

impl Config {
    pub fn fetch<P: AsRef<Path>>(path: P) -> Config {
        Self::open(path).unwrap_or(Config::default())
    }

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Config> {
        let config_file = File::open(path)?;
        serde_json::from_reader(config_file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn pick_song(&self, stage: StageId) -> Option<Song> {
        let mut rng = rand::thread_rng();
        self.playlists
            .get(&stage)
            .and_then(|songs| songs.as_slice().choose(&mut rng))
            .map(Clone::clone)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Song {
    Local(String),
}

impl Song {
    pub fn play(&self, device: &rodio::Device) -> io::Result<rodio::Sink> {
        match self {
            Song::Local(path) => {
                let file = File::open(path)?;
                rodio::play_once(&device, BufReader::new(file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))
            }
        }
    }
}
