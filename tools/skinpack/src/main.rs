use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "skinpack")]
#[command(about = "RemixClient Skin Pack CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack a skin directory into a zip archive and compute SHA-256 checksums
    Pack {
        /// Source directory containing pack.json and skins/
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Output zip file path
        #[arg(short, long, value_name = "OUT.ZIP")]
        output: PathBuf,
    },
    /// Verify pack structure, PNG signatures, dimensions, and checksums
    Verify {
        /// Path to the pack zip file
        #[arg(value_name = "PACK.ZIP")]
        pack: PathBuf,
    },
    /// Generate an index JSON file from a directory of skin packs
    Index {
        /// Directory containing multiple .zip skin packs
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Output index JSON path
        #[arg(short, long, value_name = "INDEX.JSON")]
        output: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackConfig {
    name: String,
    author: String,
    license: String,
    version: String,
    skins: Vec<SkinEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SkinEntry {
    uuid: String,
    name: String,
    model: String, // "classic" or "slim"
    skin_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cape_sha256: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SkinMeta {
    name: String,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct IndexEntry {
    filename: String,
    name: String,
    version: String,
    author: String,
    sha256: String,
    size_bytes: u64,
    skin_count: usize,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack { dir, output } => cmd_pack(&dir, &output)?,
        Commands::Verify { pack } => cmd_verify(&pack)?,
        Commands::Index { dir, output } => cmd_index(&dir, &output)?,
    }

    Ok(())
}

fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn validate_png(path: &Path) -> Result<(u32, u32)> {
    let mut file = File::open(path).with_context(|| format!("Failed to open PNG: {:?}", path))?;
    let mut header = [0u8; 24];
    let n = file.read(&mut header)?;
    if n < 24 {
        return Err(anyhow!("File too small to be a valid PNG: {:?}", path));
    }

    // PNG signature: 89 50 4E 47 0D 0A 1A 0A
    let png_sig: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if header[..8] != png_sig {
        return Err(anyhow!("Invalid PNG signature in {:?}", path));
    }

    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);

    let valid = matches!(
        (width, height),
        (64, 32) | (64, 64) | (128, 64) | (128, 128)
    );

    if !valid {
        return Err(anyhow!(
            "Invalid skin/cape dimensions {}x{} in {:?}. Allowed: 64x32, 64x64, 128x64, 128x128",
            width,
            height,
            path
        ));
    }

    Ok((width, height))
}

fn sanitize_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Skin name cannot be empty"));
    }
    if trimmed.chars().count() > 64 {
        return Err(anyhow!("Skin name exceeds 64 characters: {}", trimmed));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(anyhow!(
            "Skin name contains illegal path characters: {}",
            trimmed
        ));
    }
    Ok(trimmed.to_string())
}

fn cmd_pack(dir: &Path, output: &Path) -> Result<()> {
    let pack_json_path = dir.join("pack.json");
    if !pack_json_path.exists() {
        return Err(anyhow!("pack.json not found in {:?}", dir));
    }

    let content = fs::read_to_string(&pack_json_path)
        .with_context(|| format!("Failed to read {:?}", pack_json_path))?;
    let mut pack_config: PackConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {:?}", pack_json_path))?;

    let skins_dir = dir.join("skins");
    if !skins_dir.exists() {
        return Err(anyhow!("skins/ directory not found in {:?}", dir));
    }

    let mut skin_entries = Vec::new();

    for entry in fs::read_dir(&skins_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let uuid_str = path.file_name().unwrap().to_string_lossy().to_string();

            let skin_png = path.join("skin.png");
            if !skin_png.exists() {
                return Err(anyhow!("Missing skin.png in {:?}", path));
            }
            validate_png(&skin_png)?;
            let skin_sha = compute_sha256(&skin_png)?;

            let cape_png = path.join("cape.png");
            let cape_sha = if cape_png.exists() {
                validate_png(&cape_png)?;
                Some(compute_sha256(&cape_png)?)
            } else {
                None
            };

            let meta_json = path.join("meta.json");
            if !meta_json.exists() {
                return Err(anyhow!("Missing meta.json in {:?}", path));
            }
            let meta_content = fs::read_to_string(&meta_json)?;
            let meta: SkinMeta = serde_json::from_str(&meta_content)
                .with_context(|| format!("Failed to parse {:?}", meta_json))?;
            let clean_name = sanitize_name(&meta.name)?;

            skin_entries.push(SkinEntry {
                uuid: uuid_str,
                name: clean_name,
                model: meta.model,
                skin_sha256: skin_sha,
                cape_sha256: cape_sha,
            });
        }
    }

    pack_config.skins = skin_entries;

    let updated_pack_json = serde_json::to_string_pretty(&pack_config)?;
    fs::write(&pack_json_path, updated_pack_json)?;

    let file = File::create(output)
        .with_context(|| format!("Failed to create zip output: {:?}", output))?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let walker = WalkDir::new(dir).into_iter();
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(dir).map_err(|e| anyhow!("{}", e))?;

        if name.as_os_str().is_empty() {
            continue;
        }

        let name_str = name.to_string_lossy().replace('\\', "/");
        if name_str.contains("..") || name_str.starts_with('/') {
            return Err(anyhow!(
                "Path traversal detected in source directory: {}",
                name_str
            ));
        }

        if path.is_dir() {
            zip.add_directory(name_str, options)?;
        } else if path.is_file() {
            zip.start_file(name_str, options)?;
            let mut f = File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }

    zip.finish()?;
    println!("Successfully packed {:?} into {:?}", dir, output);
    Ok(())
}

fn cmd_verify(pack_path: &Path) -> Result<()> {
    if !pack_path.exists() {
        return Err(anyhow!("Pack zip not found: {:?}", pack_path));
    }

    let file = File::open(pack_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name();
        if name.contains("..") || name.starts_with('/') || name.contains('\\') {
            return Err(anyhow!("Path traversal detected in zip entry: {}", name));
        }
    }

    let config: PackConfig = {
        let mut pack_json_file = archive
            .by_name("pack.json")
            .map_err(|_| anyhow!("pack.json missing in zip archive"))?;
        let mut content = String::new();
        pack_json_file.read_to_string(&mut content)?;
        serde_json::from_str(&content).context("Failed to parse pack.json inside zip")?
    };

    for skin in &config.skins {
        let skin_bytes = {
            let skin_path_in_zip = format!("skins/{}/skin.png", skin.uuid);
            let mut skin_file = archive
                .by_name(&skin_path_in_zip)
                .map_err(|_| anyhow!("Missing zip entry: {}", skin_path_in_zip))?;
            let mut bytes = Vec::new();
            skin_file.read_to_end(&mut bytes)?;
            bytes
        };

        let mut hasher = Sha256::new();
        hasher.update(&skin_bytes);
        let computed_sha = hex::encode(hasher.finalize());
        if computed_sha != skin.skin_sha256 {
            return Err(anyhow!(
                "SHA-256 mismatch for skin {}: expected {}, got {}",
                skin.uuid,
                skin.skin_sha256,
                computed_sha
            ));
        }

        if let Some(ref expected_cape_sha) = skin.cape_sha256 {
            let cape_bytes = {
                let cape_path_in_zip = format!("skins/{}/cape.png", skin.uuid);
                let mut cape_file = archive
                    .by_name(&cape_path_in_zip)
                    .map_err(|_| anyhow!("Missing zip entry: {}", cape_path_in_zip))?;
                let mut cape_bytes = Vec::new();
                cape_file.read_to_end(&mut cape_bytes)?;
                cape_bytes
            };

            let mut hasher = Sha256::new();
            hasher.update(&cape_bytes);
            let computed_cape_sha = hex::encode(hasher.finalize());
            if &computed_cape_sha != expected_cape_sha {
                return Err(anyhow!(
                    "SHA-256 mismatch for cape {}: expected {}, got {}",
                    skin.uuid,
                    expected_cape_sha,
                    computed_cape_sha
                ));
            }
        }
    }

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name();
        if name.contains("..") || name.starts_with('/') || name.contains('\\') {
            return Err(anyhow!("Path traversal detected in zip entry: {}", name));
        }
    }

    println!(
        "Pack verification passed: {} skins found.",
        config.skins.len()
    );
    Ok(())
}

fn cmd_index(dir: &Path, output: &Path) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!("Directory not found: {:?}", dir));
    }

    let mut index_entries = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("zip") {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let size_bytes = fs::metadata(&path)?.len();
                    let sha256 = compute_sha256(&path)?;

                    let config = {
                        let file = File::open(&path)?;
                        let mut archive = zip::ZipArchive::new(file)?;
                        let mut pack_json_file = archive
                            .by_name("pack.json")
                            .map_err(|_| anyhow!("pack.json missing in {:?}", path))?;
                        let mut content = String::new();
                        pack_json_file.read_to_string(&mut content)?;
                        serde_json::from_str::<PackConfig>(&content)?
                    };

                    index_entries.push(IndexEntry {
                        filename,
                        name: config.name,
                        version: config.version,
                        author: config.author,
                        sha256,
                        size_bytes,
                        skin_count: config.skins.len(),
                    });
                }
            }
        }
    }

    let json = serde_json::to_string_pretty(&index_entries)?;
    fs::write(output, json)?;
    println!(
        "Successfully built index with {} packs into {:?}",
        index_entries.len(),
        output
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_valid_png(path: &Path, width: u32, height: u32) -> Result<()> {
        let mut data = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82];
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&[
            8, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 73, 68, 65, 84, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
            0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ]);
        fs::write(path, data)?;
        Ok(())
    }

    #[test]
    fn test_round_trip_pack_and_verify() -> Result<()> {
        let tmp = tempdir()?;
        let pack_dir = tmp.path().join("my_pack");
        let skins_dir = pack_dir
            .join("skins")
            .join("069a79f4-44e9-4726-a5be-fca90e38aaf5");
        fs::create_dir_all(&skins_dir)?;

        let pack_json = r#"{
            "name": "Test Pack",
            "author": "Tester",
            "license": "MIT",
            "version": "1.0",
            "skins": []
        }"#;
        fs::write(pack_dir.join("pack.json"), pack_json)?;

        let meta_json = r#"{
            "name": "Steve",
            "model": "classic"
        }"#;
        fs::write(skins_dir.join("meta.json"), meta_json)?;

        create_valid_png(&skins_dir.join("skin.png"), 64, 64)?;

        let zip_out = tmp.path().join("output.zip");
        cmd_pack(&pack_dir, &zip_out)?;
        cmd_verify(&zip_out)?;

        Ok(())
    }

    #[test]
    fn test_invalid_png_dimension() -> Result<()> {
        let tmp = tempdir()?;
        let skin_path = tmp.path().join("bad.png");
        create_valid_png(&skin_path, 100, 100)?;

        let res = validate_png(&skin_path);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Invalid skin/cape dimensions"));

        Ok(())
    }

    #[test]
    fn test_invalid_png_signature() -> Result<()> {
        let tmp = tempdir()?;
        let skin_path = tmp.path().join("bad_sig.png");
        fs::write(&skin_path, b"not a png file at all...")?;

        let res = validate_png(&skin_path);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Invalid PNG signature"));

        Ok(())
    }

    #[test]
    fn test_sha256_mismatch() -> Result<()> {
        let tmp = tempdir()?;
        let pack_dir = tmp.path().join("my_pack");
        let skins_dir = pack_dir
            .join("skins")
            .join("069a79f4-44e9-4726-a5be-fca90e38aaf5");
        fs::create_dir_all(&skins_dir)?;

        let pack_json = r#"{
            "name": "Test Pack",
            "author": "Tester",
            "license": "MIT",
            "version": "1.0",
            "skins": []
        }"#;
        fs::write(pack_dir.join("pack.json"), pack_json)?;

        let meta_json = r#"{
            "name": "Steve",
            "model": "classic"
        }"#;
        fs::write(skins_dir.join("meta.json"), meta_json)?;

        create_valid_png(&skins_dir.join("skin.png"), 64, 64)?;

        let zip_out = tmp.path().join("output.zip");
        cmd_pack(&pack_dir, &zip_out)?;

        // Tamper pack.json inside zip by creating a corrupted zip with wrong hash
        let corrupted_zip = tmp.path().join("corrupted.zip");
        let uuid_str = "069a79f4-44e9-4726-a5be-fca90e38aaf5";
        let (tampered_json, skin_bytes) = {
            let (tampered_json, bytes) = {
                let file = File::open(&zip_out)?;
                let mut archive = zip::ZipArchive::new(file)?;
                let mut content = String::new();
                {
                    let mut pack_file = archive.by_name("pack.json")?;
                    pack_file.read_to_string(&mut content)?;
                }
                let mut cfg: PackConfig = serde_json::from_str(&content)?;
                cfg.skins[0].skin_sha256 =
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string();
                let tj = serde_json::to_string(&cfg)?;

                let mut bytes = Vec::new();
                {
                    let mut skin_file = archive.by_name(&format!("skins/{}/skin.png", uuid_str))?;
                    skin_file.read_to_end(&mut bytes)?;
                }
                (tj, bytes)
            };
            (tampered_json, bytes)
        };

        {
            let out_file = File::create(&corrupted_zip)?;
            let mut zip = zip::ZipWriter::new(out_file);
            let options = zip::write::FileOptions::default();

            zip.start_file("pack.json", options)?;
            zip.write_all(tampered_json.as_bytes())?;

            zip.start_file(format!("skins/{}/skin.png", uuid_str), options)?;
            zip.write_all(&skin_bytes)?;
            zip.finish()?;
        }

        let verify_res = cmd_verify(&corrupted_zip);
        assert!(verify_res.is_err());
        assert!(verify_res
            .unwrap_err()
            .to_string()
            .contains("SHA-256 mismatch"));

        Ok(())
    }

    #[test]
    fn test_path_traversal_detection() -> Result<()> {
        let tmp = tempdir()?;
        let pack_dir = tmp.path().join("my_pack");
        let skins_dir = pack_dir
            .join("skins")
            .join("069a79f4-44e9-4726-a5be-fca90e38aaf5");
        fs::create_dir_all(&skins_dir)?;

        let pack_json = r#"{
            "name": "Test Pack",
            "author": "Tester",
            "license": "MIT",
            "version": "1.0",
            "skins": []
        }"#;
        fs::write(pack_dir.join("pack.json"), pack_json)?;
        let meta_json = r#"{"name": "Steve", "model": "classic"}"#;
        fs::write(skins_dir.join("meta.json"), meta_json)?;
        create_valid_png(&skins_dir.join("skin.png"), 64, 64)?;

        let zip_out = tmp.path().join("evil.zip");
        {
            let file = File::create(&zip_out)?;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::FileOptions::default();
            zip.start_file("../../evil.png", options)?;
            zip.write_all(b"malicious content")?;
            zip.finish()?;
        }

        let res = cmd_verify(&zip_out);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Path traversal"));

        Ok(())
    }

    #[test]
    fn test_index_command() -> Result<()> {
        let tmp = tempdir()?;
        let packs_dir = tmp.path().join("packs");
        fs::create_dir_all(&packs_dir)?;

        for i in 1..=2 {
            let pack_dir = tmp.path().join(format!("pack{}", i));
            let skins_dir = pack_dir
                .join("skins")
                .join("069a79f4-44e9-4726-a5be-fca90e38aaf5");
            fs::create_dir_all(&skins_dir)?;

            let pack_json = format!(
                r#"{{
                "name": "Pack {}",
                "author": "Tester",
                "license": "MIT",
                "version": "1.{}",
                "skins": []
            }}"#,
                i, i
            );
            fs::write(pack_dir.join("pack.json"), pack_json)?;

            let meta_json = r#"{
                "name": "Steve",
                "model": "classic"
            }"#;
            fs::write(skins_dir.join("meta.json"), meta_json)?;
            create_valid_png(&skins_dir.join("skin.png"), 64, 64)?;

            let zip_out = packs_dir.join(format!("pack{}.zip", i));
            cmd_pack(&pack_dir, &zip_out)?;
        }

        let index_out = tmp.path().join("index.json");
        cmd_index(&packs_dir, &index_out)?;

        assert!(index_out.exists());
        let index_content = fs::read_to_string(&index_out)?;
        let entries: Vec<IndexEntry> = serde_json::from_str(&index_content)?;
        assert_eq!(entries.len(), 2);

        Ok(())
    }
}
