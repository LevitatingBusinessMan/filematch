/*
Behavior:
When single dir or file is given, return md5sums of all files.
When 2 dirs or files are given, compare with builtin function
When --check is used on a file, assume that file to be a checksum listing and match it
*/

use std::env;
use std::path::Path;
use std::io;
use md5::{Md5, Digest};
use std::fs::File;
use std::io::Read;

struct Md5Entry {
    path: String,
    md5sum: String
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    
    //Show help
    if args.len() == 0 {
        println!("You forgot to mention a file");
        std::process::exit(1);
    }

    if args.len() > 2 {
        println!("Filematch currently only supports 2 directories to be compared");
        std::process::exit(1);
    }

    fn md5_file(path: &Path) -> Result<String, io::Error> {
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

    fn grab_md5s(path: &Path, vec: &Vec<Md5Entry>) -> Result<(),io::Error> {

         if !path.exists() {
            println!("Directory {} does not exist", path.to_str().unwrap());
            std::process::exit(1);
        }
        
        if path.is_dir() {
            for entry in std::fs::read_dir(path.to_str().unwrap())? {
                let entry = entry?;
                
                if entry.path().is_dir() {
                
                    grab_md5s(entry.path().as_path(), &vec).expect("Something went wrong traversing directories");
                
                } else {

                    let md5sum = md5_file(entry.path().as_path())?;
                    //canonicalize also resolves links which we don't want but whatever
                    println!("{} {}", md5sum, entry.path().canonicalize().unwrap().display());

                }

            }
        }
        else {
            let md5sum = md5_file(path)?;
            println!("{} {}", md5sum, path.canonicalize().unwrap().display());
        }
        Ok(())
    }

    //This isn't actually used yet, will prob get removed later
    //Instead this vector will only get filled when reading existing checksum lists, then used as an iter.
    let md5s = Vec::new();

    // Run md5s
    if args.len() == 1 {
        grab_md5s(Path::new(&args[0]), &md5s).expect("Something went wrong traversing directories");
    }

}
