#![allow(unused)]
pub fn extract(
    zip_file_path: &std::path::Path,
    destination_dir: &std::path::Path,
) -> zip::result::ZipResult<()> {
    // Open the ZIP file
    let file = std::fs::File::open(zip_file_path)?;
    let mut archive = zip::read::ZipArchive::new(std::io::BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            std::fs::create_dir_all(destination_dir)?;
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

pub fn extract_file(
    zip_file_path: &std::path::Path,
    archived_file_path: &std::path::Path,
    destination_filename: &std::path::Path,
) -> zip::result::ZipResult<()> {
    let file = std::fs::File::open(zip_file_path)?;
    let mut archive = zip::read::ZipArchive::new(std::io::BufReader::new(file))?;
    let mut file = archive.by_name(
        archived_file_path
            .to_str()
            .ok_or(zip::result::ZipError::FileNotFound)?,
    )?;

    if let Some(parent) = destination_filename.parent() {
        std::fs::create_dir_all(&parent)?;
    }
    let mut outfile = std::fs::File::create(&destination_filename)?;
    std::io::copy(&mut file, &mut outfile)?;
    Ok(())
}
