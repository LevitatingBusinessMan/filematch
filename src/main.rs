/*
Behavior:
When single dir or file is given, return md5sums of all files.
When 2 dirs or files are given, compare with builtin function
When --check is used on a file, assume that file to be a checksum listing and match it
There should also be a way to go to a directory, then use a checksum list like a lookup table. (--tgt folder?)
This ensures that you will also list any deleted or created directories.
*/

//TODO: relative paths

use std::env;
use std::path::Path;
use std::io;
use std::fs::File;
use std::io::Read;
use md5::{Md5, Digest};
use std::io::BufRead;
use std::collections::HashMap;
use pathdiff::diff_paths;
use std::process::exit;

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
        exit(1);
    }

    // Run md5s
    if args.len() == 1 && !check {
        make(&args[0]);
    }

    // Checklist check
    if args.len() == 1 && check {
        check_mode(&args[0]);
    }

    //Compare multiple dirs
    if args.len() > 1 {
        let dirs: Vec::<&Path> = args.iter().map(|str| Path::new(str)).collect();

        //Check for files instead of folders
        if dirs.iter().any(|dir| dir.is_file()) {
            if dirs.iter().all(|dir| dir.is_file()) {
                compare_single_files(dirs)
            } else {
                println!("Cannot compare folders to files");
                exit(1);
            }
        }
        
        else {
            let lists = create_lists(&dirs);
            compare(lists);
        }
    }

}

#[derive(Debug)]
struct Md5Entry {
    path: String,
    md5sum: String
}

fn compare_single_files(files: Vec<&Path>) {

    let mut hashes: Vec::<String> = vec!();
    let mut different = false;

    for file in &files {
        if !file.exists() {
            println!("File {} doesn't exist", file.to_str().unwrap());
            exit(1);
        }

        let hash = md5_file(file).unwrap();
        
        if let Some(prev_hash) = hashes.last() {
            if prev_hash != &hash {
                different = true;
            }
        }

        hashes.push(hash);
    }
    
    if different {
        println!("{} ({})",
            files.iter().map(|path| path.file_name().unwrap().to_str().unwrap()).collect::<Vec<&str>>().join(","),
            hashes.join(" | ")
        );
    }

}

fn create_lists(dirs: &Vec<&Path>) -> Vec::<HashMap::<String,String>> {
    let mut lists = Vec::with_capacity(dirs.len());
    for dir in dirs {

        let mut files = HashMap::new();

        traverser(dir, &mut |path: &Path| {
            let md5 = md5_file(path).unwrap();
            let rel_path = diff_paths(path,dir).unwrap();
            files.insert(rel_path.to_str().unwrap().to_owned(), md5);
        }).expect("Something went wrong traversing the directories.");
        lists.push(files);
    }

    return lists;

}

///Compare multiple filelists
fn compare(lists: Vec::<HashMap::<String,String>>) {
    
    //All files already compared
    let mut compared = vec!();
    
    for files in &lists {
        for (path, md5) in files {
            //Don't compare stuff twice
            if compared.iter().any(|path_: &String| path_ == path) {continue};

            let mut different = false;
            let mut hashes = vec!();

            for files_ in &lists {
                match files_.get(path) {
                    Some(hash) => {
                        hashes.push(hash.to_owned());
                        if hash != md5 {
                            different = true;
                        }
                    },
                    None => {
                        different = true;
                        hashes.push("non-exist".to_owned())
                    }
                };
            }
            
            if different {
                println!("{} ({})", path, hashes.join(" | "));
            }

            //If I could make this a reference things would be more memory efficient
            compared.push(path.to_owned());
        }
    }

}

fn get_checklist<P: AsRef<Path>>(path: P) -> impl Iterator<Item = Md5Entry> {

    let f = File::open(path).expect("Error opening file");
    let f = io::BufReader::new(f);
    
    f.lines().map(|line| {
        let line = line.unwrap();
        let line_split: Vec<&str> = line.split_whitespace().collect();
        
        if line_split.len() != 2 {
            println!("Invalid checksum file");
            exit(1);
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
        exit(1);
    }

    if !path.is_file() {
        println!("{} is a directory", dir);
        exit(1);
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

    traverser(Path::new(dir), &mut callback).expect("Something went wrong traversing directories");
}

///Traverses a path and runs the provided callback for each file
fn traverser(path: &Path, cb: &mut impl FnMut(&Path)) -> Result<(),io::Error> {

    if !path.exists() {
        println!("Directory {} does not exist", path.to_str().unwrap());
        exit(1);
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

/*
Currently this always returns a string.
It returns "deteled" when a file is deleted.
In the future it's probably better if instead it returns an enum instead.
*/
fn md5_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut hasher = Md5::new();
    let f = File::open(path);
    let mut buffer: [u8; 1024] = [0; 1024];

    let mut f = match f {
        Ok(file) => file,
        Err(_file) => return Ok("deleted".to_owned())
    };

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
