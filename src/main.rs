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

fn remove_duplicates(directory: &str) -> io::Result<()> {
    let mut checksums: HashMap<String, String> = HashMap::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            match compute_sha256(path) {
                Ok(sha256) => {
                    if let Some(existing_path) = checksums.get(&sha256) {
                        println!("Usuwanie duplikatu: {} (taki sam jak {})", path.display(), existing_path);
                        fs::remove_file(path)?;
                    } else {
                        checksums.insert(sha256, path.to_string_lossy().to_string());
                    }
                }
                Err(err) => eprintln!("Błąd obliczania SHA-256 dla {}: {}", path.display(), err),
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Użycie: {} /ścieżka/do/katalogu", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];
    if let Err(err) = remove_duplicates(directory) {
        eprintln!("Błąd: {}", err);
    }
}