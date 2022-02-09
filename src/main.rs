use std::path::Path;
use std::env;

fn main() {

    let rom_path = env::args().nth(1).expect("No ROM path provided");
    let rom_path = Path::new(&rom_path);
    if !(file_path.exists() && (file_path.extension().unwrap() == "gb")) {
        panic!("Invalid ROM file");
    }

    

}

fn verify_path(path: &String) -> &Path {
    let file_path = Path::new(path);
    file_path.exists() && (file_path.extension().unwrap() == "gb") // TODO: change extension.
}
