#![feature(iterator_try_collect, file_create_new)]
use std::{fs, io::{BufWriter, Write, BufReader, Read}};

use num::integer::Roots;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    files: Vec<String>,
    result: String,
}

fn load_config() -> Config {
    let path = "config.ron";
    let config_str = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Couldn't load config: {e}");

            let config = Config::default();

            if e.kind() == std::io::ErrorKind::NotFound {
                let _ = fs::write(path, to_string_pretty(&config, PrettyConfig::default()).unwrap());
            }

            return config;
        }
    };

    match ron::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse config: {e}");

            Config::default()
        },
    }
}

fn main() {
    let config = load_config();

    if config.files.is_empty() {
        eprintln!("There has to be at least one file to combine.");
        return;
    }
    let side_len = config.files.len().sqrt();
    if side_len.pow(2) != config.files.len() {
        eprintln!("The number of files has to be a square number.");
        return;
    }

    let files = config.files.iter().map(|path| fs::File::open(path)).try_collect::<Vec<_>>().unwrap();

    let get_side_len = |file: &fs::File| {
        let cells = file.metadata().unwrap().len() / 2;
        let unit_sz = cells.sqrt();
        (unit_sz.pow(2) == cells).then_some(unit_sz).expect("Expected hgt file to be a square.") as usize
    };

    let unit_sz = get_side_len(&files[0]);

    if !files[1..].iter().all(|file| get_side_len(file) == unit_sz) {
        eprintln!("Expected all part maps to be the same size.");
    }

    let mut buf_readers = files.into_iter().map(BufReader::new).collect::<Vec<_>>();

    let result = fs::File::create_new(config.result).unwrap();
    let mut result = BufWriter::new(result);

    let res_size = unit_sz * side_len;

    let mut row = vec![0u8; res_size * 2];

    for y in 0..res_size {
        let i_y = y / unit_sz;
        for i_x in 0..side_len {
            buf_readers[i_y * side_len + i_x].read(&mut row[i_x * unit_sz * 2..(i_x + 1) * unit_sz * 2]).expect("We should be in bounds here");
        }
        result.write(&row).unwrap();
    }
}
