use std::path::Path;
use std::env;

fn main() {
    let cartridge_path = env::args().nth(1).expect("No ROM path provided");
    if !verify_path(&cartridge_path) {
        panic!("Invalid path");
    }

}

fn verify_path(path: &String) -> bool {
    let file_path = Path::new(path);
    file_path.exists() && (file_path.extension().unwrap() == "gb") // TODO: change extension.
}
