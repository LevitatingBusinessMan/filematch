/*
Behavior:
When single dir or file is given, return md5sums of all files.
When 2 dirs or files are given, compare with builtin function
When --check is used on a file, assume that file to be a checksum listing and match it
There should also be a way to go to a directory, then use a checksum list like a lookup table.
This ensures that you will also list any deleted or created directories.
*/

use std::env;
use std::path::Path;
use std::io;
use std::fs::File;
use std::io::Read;
use md5::{Md5, Digest};
use std::io::BufRead;

fn main() {
    //let args: Vec<String> = env::args().skip(1).collect();
    let mut args = vec![];
    
    let mut check = false;

    for argument in  env::args().skip(1) {
        match argument.as_str() {
            "--check" => check = true,
            _ => args.push(argument)
        }
    }

    //Show help
    if args.len() == 0 {
        println!("You forgot to mention a file");
        std::process::exit(1);
    }

    if args.len() > 2 {
        println!("Filematch currently only supports 2 directories to be compared");
        std::process::exit(1);
    }

    //This isn't actually used yet, will prob get removed later
    //Instead this vector will only get filled when reading existing checksum lists, then used as an iter.
    //let md5s = Vec::new();

    //https://docs.rs/same-file/1.0.6/same_file/fn.is_same_file.html

    // Run md5s
    if args.len() == 1 && !check {
        make(&args[0]);
    }

    if args.len() == 1 && check {
        check_mode(&args[0]);
    }

}

#[derive(Debug)]
struct Md5Entry {
    path: String,
    md5sum: String
}

/*  fn lookup_(checklist: &String, dir: &String) {
    let checklist = Path::new(checklist);
    let path = Path::new(dir);

    if !checklist.exists() {
        println!("File {} does not exist", dir);
        std::process::exit(1);
    }

    if !path.exists() {
        println!("File {} does not exist", dir);
        std::process::exit(1);
    }

    if !checklist.is_file() {
        println!("{} is a directory", dir);
        std::process::exit(1);
    }

    let checklist = get_checklist(path).collect();

} */

fn get_checklist<P: AsRef<Path>>(path: P) -> impl Iterator<Item = Md5Entry> {

    let f = File::open(path).expect("Error opening file");
    let f = io::BufReader::new(f);
    
    f.lines().map(|line| {
        let line = line.unwrap();
        let line_split: Vec<&str> = line.split_whitespace().collect();
        
        if line_split.len() != 2 {
            println!("Invalid checksum file");
            std::process::exit(1);
        }

        let old_checksum = line_split[0];
        let file = line_split[1];

        Md5Entry {
            path: file.to_string(),
            md5sum: old_checksum.to_string()
        }
    })
}

fn check_mode(dir: &String) {
    let path = Path::new(dir);
    
    if !path.exists() {
        println!("File {} does not exist", dir);
        std::process::exit(1);
    }

    if !path.is_file() {
        println!("{} is a directory", dir);
        std::process::exit(1);
    }
    
    for checksum in get_checklist(path) {
        let old_checksum = checksum.md5sum;
        let file = checksum.path;

        let new_checksum = md5_file(&file).unwrap();
        
        if old_checksum != new_checksum {
            println!("{} ({} -> {})", file, old_checksum, new_checksum);
        }
    }

}

fn make(dir: &String) {

    fn callback(path: &Path) {
        let md5sum = md5_file(path).expect("Something went wrong creating md5sum");
        //canonicalize also resolves links which we don't want but whatever
        println!("{} {}", md5sum, path.canonicalize().unwrap().display());
    }

    traverser(Path::new(dir), &callback).expect("Something went wrong traversing directories");
}

fn traverser(path: &Path, cb: &Fn(&Path)) -> Result<(),io::Error> {

    if !path.exists() {
        println!("Directory {} does not exist", path.to_str().unwrap());
        std::process::exit(1);
    }
    
    if path.is_dir() {
        for entry in std::fs::read_dir(path.to_str().unwrap())? {
            let entry = entry?;
            
            if entry.path().is_dir() {
            
                traverser(entry.path().as_path(), cb).expect("Something went wrong traversing directories");
            
            } else {

                cb(entry.path().as_path())

            }

        }
    }
    else {
        cb(path);
    }
    Ok(())
}

fn md5_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut hasher = Md5::new();
    let mut f = File::open(path).expect("Error opening file");
    let mut buffer: [u8; 1024] = [0; 1024];

    loop {
        let count = f.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let bytearray = hasher.finalize();
    let hexstring = hex::encode(bytearray);
    Ok(hexstring)

}
