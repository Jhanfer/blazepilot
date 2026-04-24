use std::path::{Path, PathBuf};
use crate::core::files::{file_extension::{ArchiveType, FileExtension}, motor::FileEntry};
use thiserror::Error;

//Manejo de errores
#[derive(Debug, Error)]
pub enum ZipError {
    #[error("")]
    Zip(#[from] zip::result::ZipError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Formato de archivo no soportado")]
    UnsupportedFormat(PathBuf),
}

pub type ZipResult<T> = Result<T, ZipError>;



pub trait ArchiveExtractor {
    fn extract(&self, archive: &Path, dest: &Path) -> ZipResult<()>;
}

pub struct ZipExtractor;


impl ArchiveExtractor for ZipExtractor {
    fn extract(&self, archive: &Path, dest: &Path) -> ZipResult<()> {
        let file = std::fs::File::open(archive)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest.join(file.name());

            if !outpath.starts_with(dest) {
                continue;
            }

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}



pub struct TarExtractor{
    kind: ArchiveType,
}

impl ArchiveExtractor for TarExtractor {
    fn extract(&self, archive: &Path, dest: &Path) -> ZipResult<()> {
        let file = std::fs::File::open(archive)?;

        let reader: Box<dyn std::io::Read> = match self.kind {
            ArchiveType::TarGz => Box::new(flate2::read::GzDecoder::new(file)),
            ArchiveType::TarXz => Box::new(xz2::read::XzDecoder::new(file)),
            ArchiveType::TarBz2 => Box::new(flate2::read::GzDecoder::new(file)),
            ArchiveType::Tar => Box::new(file),
            _ => unreachable!(),
        };

        let mut archive = tar::Archive::new(reader);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let outpath = dest.join(&path);

            if !outpath.starts_with(dest) {
                continue;
            }

            entry.unpack(&outpath)?;
        }

        Ok(())
    }
}



pub struct GzExtractor;

impl ArchiveExtractor for GzExtractor {
    fn extract(&self, archive: &Path, dest: &Path) -> ZipResult<()> {
        let file = std::fs::File::open(archive)?;
        let mut decoder = flate2::read::GzDecoder::new(file);

        let output_name = archive
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let output = dest.join(output_name);

        let mut out = std::fs::File::create(output)?;
        std::io::copy(&mut decoder, &mut out)?;

        Ok(())
    }
}

pub struct ZipManager;

impl Default for ZipManager { 
    fn default() -> Self { 
        Self {} 
    } 
}

impl ZipManager {
    pub fn new() -> Self { Self::default() }

    fn extractor_for(ext: &FileExtension) -> ZipResult<Box<dyn ArchiveExtractor>> {
        match ext {
            FileExtension::Archive(ArchiveType::Zip) => {
                Ok(Box::new(ZipExtractor))
            },

            FileExtension::Archive(t @ ArchiveType::Tar)
            | FileExtension::Archive(t @ ArchiveType::TarGz)
            | FileExtension::Archive(t @ ArchiveType::TarXz)
            | FileExtension::Archive(t @ ArchiveType::TarBz2) => {
                Ok(Box::new(TarExtractor { kind: t.clone() }))
            }

            FileExtension::Archive(ArchiveType::Gz) => {
                Ok(Box::new(GzExtractor))
            }


            //-- No soportados por el momento ------------
            FileExtension::Archive(ArchiveType::Rar)
            | FileExtension::Archive(ArchiveType::SevenZ)
            | FileExtension::Archive(ArchiveType::Zst)
            | FileExtension::Archive(ArchiveType::Bz2)
            | FileExtension::Archive(ArchiveType::Xz) => {
                Err(ZipError::UnsupportedFormat(PathBuf::new()))
            }

            _ => Err(ZipError::UnsupportedFormat(PathBuf::new()))
        }
    }


    pub fn extract(&self, entry: &FileEntry, dest: &Path) -> ZipResult<()> {
        self.assert_archive(entry)?;

        let extractor = Self::extractor_for(&entry.extension)?;
        extractor.extract(&entry.full_path, dest)
    }


    fn assert_archive(&self, entry: &FileEntry) -> ZipResult<()> { 
        match &entry.extension { 
            FileExtension::Archive(_) => Ok(()),
            _ => Err(ZipError::UnsupportedFormat(entry.full_path.clone())) 
        } 
    }
}
