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

fn process_duplicates(directory: &str, target_dir: Option<&str>) -> io::Result<()> {
    let mut checksums: HashMap<String, String> = HashMap::new();

    let target_path = if let Some(target_path) = target_dir {
        let target_path = Path::new(target_path);
        if !target_path.exists() {
            fs::create_dir_all(target_path)?;
        }
        Some(target_path)
    } else {
        None
    };

    for entry in WalkDir::new(directory).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_dir() {
            println!("Przetwarzanie folderu: {}", path.display());
        }

        if path.is_file() {
            println!("Skanowanie pliku: {}", path.display());

            match compute_sha256(path) {
                Ok(sha256) => {
                    if let Some(existing_path) = checksums.get(&sha256) {
                        println!("\x1b[31mZduplikowany plik: {}\x1b[0m", path.display());
                        if path.to_string_lossy().len() < existing_path.len() {
                            let relative_path = path.strip_prefix(directory).unwrap_or(path);
                            let new_path = target_path.unwrap().join(relative_path);

                            if let Some(parent) = new_path.parent() {
                                fs::create_dir_all(parent)?;
                            }

                            fs::rename(path, new_path)?;
                            checksums.insert(sha256, path.to_string_lossy().to_string());
                        } else {
                            fs::remove_file(path)?;
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
        eprintln!("Użycie: {} <komenda> [opcje]", args[0]);
        eprintln!("Dostępne komendy:");
        eprintln!("  dupes [-r] <katalog> [katalog_docelowy]  - Znajduje i przenosi/usuwa duplikaty");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "dupes" => {
            if args.len() < 3 {
                eprintln!("Musisz podać katalog do przeszukania.");
                std::process::exit(1);
            }

            let (directory, target_directory) = if args[2] == "-r" {
                if args.len() < 4 {
                    eprintln!("Musisz podać katalog do przeszukania.");
                    std::process::exit(1);
                }
                (&args[3], None)
            } else {
                if args.len() < 4 {
                    eprintln!("Musisz podać katalog do przeszukania i katalog docelowy.");
                    std::process::exit(1);
                }
                (&args[2], Some(&args[3]))
            };

            if let Err(err) = process_duplicates(directory, target_directory.map(|s| s.as_str())) {
                eprintln!("Błąd: {}", err);
            }
        }
        "rename" => {
            println!("Funkcja 'rename' nie jest jeszcze zaimplementowana.");
        }
        _ => {
            eprintln!("Nieznana komenda: {}", args[1]);
            std::process::exit(1);
        }
    }
}