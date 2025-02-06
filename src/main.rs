use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};

fn compute_sha256(file_path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash_result = hasher.finalize();
    Ok(format!("{:x}", hash_result))
}

fn process_duplicates(directory: &str, target_dir: Option<&str>, remove: bool) -> io::Result<()> {
    let mut checksums: HashMap<String, String> = HashMap::new();

    if let Some(target_path) = target_dir {
        let target_path = Path::new(target_path);
        if !target_path.exists() {
            fs::create_dir_all(target_path)?;
        }
    }

    for entry in WalkDir::new(directory).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            match compute_sha256(path) {
                Ok(sha256) => {
                    if checksums.contains_key(&sha256) {
                        if remove {
                            fs::remove_file(path)?;
                        } else if let Some(target_path) = target_dir {
                            let relative_path = path.strip_prefix(directory).unwrap_or(path);
                            let new_path = Path::new(target_path).join(relative_path);

                            if let Some(parent) = new_path.parent() {
                                fs::create_dir_all(parent)?;
                            }

                            fs::rename(path, new_path)?;
                        }
                    } else {
                        checksums.insert(sha256, path.to_string_lossy().to_string());
                    }
                }
                Err(_) => {}
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Użycie: {} [-r] /ścieżka/do/katalogu [ścieżka/do/katalogu_docelowego]", args[0]);
        std::process::exit(1);
    }

    let (remove_mode, directory, target_directory) = if args[1] == "-r" {
        if args.len() < 3 {
            eprintln!("Musisz podać katalog do przeszukania.");
            std::process::exit(1);
        }
        (true, &args[2], None)
    } else {
        if args.len() < 3 {
            eprintln!("Musisz podać katalog do przeszukania i katalog docelowy.");
            std::process::exit(1);
        }
        (false, &args[1], Some(&args[2]))
    };

    if let Err(err) = process_duplicates(directory, target_directory.map(|s| s.as_str()), remove_mode) {
        eprintln!("Błąd: {}", err);
    }
}