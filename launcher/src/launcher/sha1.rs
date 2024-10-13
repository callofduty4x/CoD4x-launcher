use sha1::{Digest, Sha1};
use std::io::{BufReader, Read};

pub fn digest(path: &std::path::Path) -> std::io::Result<String> {
    let input = std::fs::File::open(path)?;
    let mut reader = BufReader::new(input);

    let digest = {
        let mut hasher = Sha1::new();
        let mut buffer = [0; 1024];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    Ok(hex::encode(digest))
}
