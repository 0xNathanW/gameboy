use std::{path::PathBuf, io::{ErrorKind, Read}, fs::File};

pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;

fn load_save(save_path: &PathBuf, ram_size: usize) -> Vec<u8> {
    match File::open(save_path) {
        Ok(mut file) => {
            let mut ram = vec![];
            file.read_to_end(&mut ram).unwrap();
            ram
        },
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            vec![0; ram_size]
        },
        Err(..) => panic!("could not read file"),
    }
}