//Trait para pasar lo que necesito a str
pub trait StrExtension {
    fn extension(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocType {
    Pdf,
    Doc,
    Docx,
    Xls,
    Xlsx,
    Ppt,
    Pptx,
    Txt,
    Md,
    Rtf,
    Csv,
    Log,
    Odt,
    Ods,
    Odp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageType {
    Png,
    Jpg,
    Gif,
    Webp,
    Bmp,
    Tiff,
    Svg,
    Ico,
    Avif,
    Heic,
}

impl StrExtension for ImageType {
    fn extension(&self) -> &'static str {
        match self {
            ImageType::Png => "png",
            ImageType::Jpg => "jpg",
            ImageType::Gif => "gif",
            ImageType::Webp => "webp",
            ImageType::Bmp => "bmp",
            ImageType::Tiff => "tiff",
            ImageType::Svg => "svg",
            ImageType::Ico => "ico",
            ImageType::Avif => "avif",
            ImageType::Heic => "heic",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoType {
    Mp4,
    Mkv,
    Avi,
    Mov,
    Wmv,
    Flv,
    Webm,
    M4v,
}

impl StrExtension for VideoType {
    fn extension(&self) -> &'static str {
        match self {
            VideoType::Mp4 => "mp4",
            VideoType::Mkv => "mkv",
            VideoType::Avi => "avi",
            VideoType::Mov => "mov",
            VideoType::Wmv => "wmv",
            VideoType::Flv => "flv",
            VideoType::Webm => "webm",
            VideoType::M4v => "m4v",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioType {
    Mp3,
    Wav,
    Flac,
    Ogg,
    Aac,
    M4a,
    Opus,
    Wma,
}

impl StrExtension for AudioType {
    fn extension(&self) -> &'static str {
        match self {
            AudioType::Mp3 => "mp3s",
            AudioType::Wav => "wav",
            AudioType::Flac => "flac",
            AudioType::Ogg => "ogg",
            AudioType::Aac => "aac",
            AudioType::M4a => "m4a",
            AudioType::Opus => "opus",
            AudioType::Wma => "wma",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveType {
    Zip,
    Tar,
    TarGz,
    TarXz,
    TarBz2,
    Gz,
    Bz2,
    Xz,
    Rar,
    SevenZ,
    #[allow(unused)]
    Zst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeType {
    Rs,
    Py,
    Js,
    Ts,
    C,
    Cpp,
    H,
    Hpp,
    Go,
    Java,
    Kt,
    Swift,
    Rb,
    Php,
    Html,
    Css,
    Scss,
    Json,
    Toml,
    Yaml,
    Xml,
    Sh,
    Bash,
    Fish,
    Zsh,
    Sql,
    R,
    Lua,
    Dart,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontType {
    Ttf,
    Otf,
    Woff,
    Woff2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableType {
    //Linux
    AppImage,
    Deb,
    Rpm,
    //Windows
    Exe,
    Msi,
    //Macos
    Dmg,
    App,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileExtension {
    Document(DocType),
    Image(ImageType),
    Video(VideoType),
    Audio(AudioType),
    Archive(ArchiveType),
    Code(CodeType),
    Font(FontType),
    Executable(ExecutableType),

    #[default]
    Unknown,
}

impl FileExtension {
    pub fn from_path(path: &std::path::Path) -> Self {
        let path_str = path.to_string_lossy().to_ascii_lowercase();

        if path_str.ends_with(".tar.gz") {
            return Self::Archive(ArchiveType::TarGz);
        }
        if path_str.ends_with(".tar.xz") {
            return Self::Archive(ArchiveType::TarXz);
        }
        if path_str.ends_with(".tar.bz2") {
            return Self::Archive(ArchiveType::TarBz2);
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let ext = if file_name.starts_with('.') {
            file_name.find('.').and_then(|i| {
                let after = &file_name[i + 1..];
                if after.is_empty() { None } else { Some(after) }
            })
        } else {
            path.extension().and_then(|e| e.to_str())
        };

        let ext = ext.map(|e| e.to_ascii_lowercase());

        match ext.as_deref() {
            Some("pdf") => Self::Document(DocType::Pdf),
            Some("doc") => Self::Document(DocType::Doc),
            Some("docx") => Self::Document(DocType::Docx),
            Some("xls") => Self::Document(DocType::Xls),
            Some("xlsx") => Self::Document(DocType::Xlsx),
            Some("ppt") => Self::Document(DocType::Ppt),
            Some("pptx") => Self::Document(DocType::Pptx),
            Some("txt") => Self::Document(DocType::Txt),
            Some("md") => Self::Document(DocType::Md),
            Some("rtf") => Self::Document(DocType::Rtf),
            Some("csv") => Self::Document(DocType::Csv),
            Some("log") => Self::Document(DocType::Log),
            Some("odt") => Self::Document(DocType::Odt),
            Some("ods") => Self::Document(DocType::Ods),
            Some("odp") => Self::Document(DocType::Odp),

            Some("png") => Self::Image(ImageType::Png),
            Some("jpg") | Some("jpeg") => Self::Image(ImageType::Jpg),
            Some("gif") => Self::Image(ImageType::Gif),
            Some("webp") => Self::Image(ImageType::Webp),
            Some("bmp") => Self::Image(ImageType::Bmp),
            Some("tiff") | Some("tif") => Self::Image(ImageType::Tiff),
            Some("svg") => Self::Image(ImageType::Svg),
            Some("ico") => Self::Image(ImageType::Ico),
            Some("avif") => Self::Image(ImageType::Avif),
            Some("heic") => Self::Image(ImageType::Heic),

            Some("mp4") => Self::Video(VideoType::Mp4),
            Some("mkv") => Self::Video(VideoType::Mkv),
            Some("avi") => Self::Video(VideoType::Avi),
            Some("mov") => Self::Video(VideoType::Mov),
            Some("wmv") => Self::Video(VideoType::Wmv),
            Some("flv") => Self::Video(VideoType::Flv),
            Some("webm") => Self::Video(VideoType::Webm),
            Some("m4v") => Self::Video(VideoType::M4v),

            Some("mp3") => Self::Audio(AudioType::Mp3),
            Some("wav") => Self::Audio(AudioType::Wav),
            Some("flac") => Self::Audio(AudioType::Flac),
            Some("ogg") => Self::Audio(AudioType::Ogg),
            Some("aac") => Self::Audio(AudioType::Aac),
            Some("m4a") => Self::Audio(AudioType::M4a),
            Some("opus") => Self::Audio(AudioType::Opus),
            Some("wma") => Self::Audio(AudioType::Wma),

            Some("zip") => Self::Archive(ArchiveType::Zip),
            Some("tar") => Self::Archive(ArchiveType::Tar),
            Some("gz") => Self::Archive(ArchiveType::Gz),
            Some("bz2") => Self::Archive(ArchiveType::Bz2),
            Some("xz") => Self::Archive(ArchiveType::Xz),
            Some("7z") => Self::Archive(ArchiveType::SevenZ),
            Some("rar") => Self::Archive(ArchiveType::Rar),

            Some("rs") => Self::Code(CodeType::Rs),
            Some("py") => Self::Code(CodeType::Py),
            Some("js") => Self::Code(CodeType::Js),
            Some("ts") => Self::Code(CodeType::Ts),
            Some("c") => Self::Code(CodeType::C),
            Some("cpp") => Self::Code(CodeType::Cpp),
            Some("h") => Self::Code(CodeType::H),
            Some("hpp") => Self::Code(CodeType::Hpp),
            Some("go") => Self::Code(CodeType::Go),
            Some("java") => Self::Code(CodeType::Java),
            Some("kt") => Self::Code(CodeType::Kt),
            Some("swift") => Self::Code(CodeType::Swift),
            Some("rb") => Self::Code(CodeType::Rb),
            Some("php") => Self::Code(CodeType::Php),
            Some("html") => Self::Code(CodeType::Html),
            Some("css") => Self::Code(CodeType::Css),
            Some("scss") => Self::Code(CodeType::Scss),
            Some("json") => Self::Code(CodeType::Json),
            Some("toml") => Self::Code(CodeType::Toml),
            Some("yaml") | Some("yml") => Self::Code(CodeType::Yaml),
            Some("xml") => Self::Code(CodeType::Xml),
            Some("sh") => Self::Code(CodeType::Sh),
            Some("bash") => Self::Code(CodeType::Bash),
            Some("fish") => Self::Code(CodeType::Fish),
            Some("zsh") => Self::Code(CodeType::Zsh),
            Some("sql") => Self::Code(CodeType::Sql),
            Some("r") => Self::Code(CodeType::R),
            Some("lua") => Self::Code(CodeType::Lua),
            Some("dart") => Self::Code(CodeType::Dart),

            Some("ttf") => Self::Font(FontType::Ttf),
            Some("otf") => Self::Font(FontType::Otf),
            Some("woff") => Self::Font(FontType::Woff),
            Some("woff2") => Self::Font(FontType::Woff2),

            Some("deb") => Self::Executable(ExecutableType::Deb),
            Some("rpm") => Self::Executable(ExecutableType::Rpm),
            Some("appimage") => Self::Executable(ExecutableType::AppImage),
            Some("exe") => Self::Executable(ExecutableType::Exe),
            Some("msi") => Self::Executable(ExecutableType::Msi),
            Some("dmg") => Self::Executable(ExecutableType::Dmg),
            Some("app") => Self::Executable(ExecutableType::App),

            _ => Self::Unknown,
        }
    }

    pub fn from_mime(mime: &str) -> Self {
        match mime {
            m if m.trim().to_lowercase().starts_with("image/") => match m.trim().to_lowercase() {
                png if png.contains("png") => Self::Image(ImageType::Png),
                jpg if jpg.contains("jpg") => Self::Image(ImageType::Jpg),
                gif if gif.contains("gif") => Self::Image(ImageType::Gif),
                webp if webp.contains("webp") => Self::Image(ImageType::Webp),
                bmp if bmp.contains("bmp") => Self::Image(ImageType::Bmp),
                tiff if tiff.contains("tiff") => Self::Image(ImageType::Tiff),
                svg if svg.contains("svg") => Self::Image(ImageType::Svg),
                ico if ico.contains("ico") => Self::Image(ImageType::Ico),
                avif if avif.contains("avif") => Self::Image(ImageType::Avif),
                heic if heic.contains("heic") => Self::Image(ImageType::Heic),

                _ => Self::Unknown,
            },

            m if m.trim().to_lowercase().starts_with("video/") => match m.trim().to_lowercase() {
                mp4 if mp4.contains("mp4") => Self::Video(VideoType::Mp4),
                mkv if mkv.contains("mkv") => Self::Video(VideoType::Mkv),
                avi if avi.contains("avi") => Self::Video(VideoType::Avi),
                mov if mov.contains("mov") => Self::Video(VideoType::Mov),
                wmv if wmv.contains("wmv") => Self::Video(VideoType::Wmv),
                flv if flv.contains("flv") => Self::Video(VideoType::Flv),
                webm if webm.contains("webm") => Self::Video(VideoType::Webm),
                m4v if m4v.contains("m4v") => Self::Video(VideoType::M4v),

                _ => Self::Unknown,
            },

            m if m.trim().to_lowercase().starts_with("audio/") => match m.trim().to_lowercase() {
                mp3 if mp3.contains("mp3") => Self::Audio(AudioType::Mp3),
                wav if wav.contains("wav") => Self::Audio(AudioType::Wav),
                flac if flac.contains("flac") => Self::Audio(AudioType::Flac),
                ogg if ogg.contains("ogg") => Self::Audio(AudioType::Ogg),
                aac if aac.contains("aac") => Self::Audio(AudioType::Aac),
                m4a if m4a.contains("m4a") => Self::Audio(AudioType::M4a),
                opus if opus.contains("opus") => Self::Audio(AudioType::Opus),
                wma if wma.contains("wma") => Self::Audio(AudioType::Wma),

                _ => Self::Unknown,
            },

            _ => Self::Unknown,
        }
    }
}

impl FileExtension {
    pub fn is_archive(&self) -> bool {
        matches!(self, FileExtension::Archive(_))
    }

    pub fn is_image(&self) -> bool {
        matches!(self, FileExtension::Image(_))
    }

    pub fn is_video(&self) -> bool {
        matches!(self, FileExtension::Video(_))
    }

    pub fn is_audio(&self) -> bool {
        matches!(self, FileExtension::Audio(_))
    }

    #[allow(unused)]
    pub fn is_code(&self) -> bool {
        matches!(self, FileExtension::Code(_))
    }
    #[allow(unused)]
    pub fn archive_type(&self) -> Option<ArchiveType> {
        match self {
            FileExtension::Archive(t) => Some(t.clone()),
            _ => None,
        }
    }
}

pub fn sniff_magic_bytes(bytes: &[u8]) -> Option<FileExtension> {
    match bytes {
        // _-_-_- imágenes _-_-_-

        //png
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => {
            Some(FileExtension::Image(ImageType::Png))
        }
        //jpg
        [0xFF, 0xD8, 0xFF, ..] => {
            Some(FileExtension::Image(ImageType::Jpg))
        }
        //gif
        [0x47, 0x49, 0x46, 0x38, ..] => {
            Some(FileExtension::Image(ImageType::Gif))
        }

        // riff / webp
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50, ..] => {
            Some(FileExtension::Image(ImageType::Webp))
        }

        // bmp
        [0x42, 0x4D, ..] => {
            Some(FileExtension::Image(ImageType::Bmp))
        }

        // tiff little endian
        [0x49, 0x49, 0x2A, 0x00, ..] => {
            Some(FileExtension::Image(ImageType::Tiff))
        }

        // tiff big endian
        [0x4D, 0x4D, 0x00, 0x2A, ..] => {
            Some(FileExtension::Image(ImageType::Tiff))
        }

        // ico
        [0x00, 0x00, 0x01, 0x00, ..] => {
            Some(FileExtension::Image(ImageType::Ico))
        }

        //avif (iso bmff)
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66, ..]
        | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x73, ..] => {
            Some(FileExtension::Image(ImageType::Avif))
        }

        //heic (iso bmff)
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63, ..]
        | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x78, ..]
        | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x69, 0x66, 0x31, ..] => {
            Some(FileExtension::Image(ImageType::Heic))
        }

        // _-_-_- vídeo _-_-_- 

        // m4a
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x4D, 0x34, 0x41, 0x20, ..] => {
            Some(FileExtension::Audio(AudioType::M4a))
        }

        // mkv / webm
        [0x1A, 0x45, 0xDF, 0xA3, ..] => {
            // No se puede distinguir de WebM solo con los primeros bytes.
            Some(FileExtension::Video(VideoType::Mkv))
        }

        // riff / avi
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x41, 0x56, 0x49, 0x20, ..] => {
            Some(FileExtension::Video(VideoType::Avi))
        }

        // mov
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x71, 0x74, 0x20, 0x20, ..] => {
            Some(FileExtension::Video(VideoType::Mov))
        }

        // m4v
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x4D, 0x34, 0x56, 0x20, ..]
        | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x4D, 0x34, 0x56, 0x48, ..] => {
            Some(FileExtension::Video(VideoType::M4v))
        }

        // mp4
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, ..] => {
            Some(FileExtension::Video(VideoType::Mp4))
        }

        // flv
        [0x46, 0x4C, 0x56, 0x01, ..] => {
            Some(FileExtension::Video(VideoType::Flv))
        }

        // wmc - mwa no se puede detectar solo con magic bytes
        [
            0x30, 0x26, 0xB2, 0x75,
            0x8E, 0x66, 0xCF, 0x11,
            0xA6, 0xD9, 0x00, 0xAA,
            0x00, 0x62, 0xCE, 0x6C,
            ..
        ] => {
            Some(FileExtension::Video(VideoType::Wmv))
        }

        // _-_-_- audio _-_-_- 

        // mp3 id3
        [0x49, 0x44, 0x33, ..]
        // mp3 frame
        | [0xFF, 0xFB, ..]
        | [0xFF, 0xF3, ..]
        | [0xFF, 0xF2, ..] => {
            Some(FileExtension::Audio(AudioType::Mp3))
        }

        // riff / wave
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x41, 0x56, 0x45, ..] => {
            Some(FileExtension::Audio(AudioType::Wav))
        }

        // flac
        [0x66, 0x4C, 0x61, 0x43, ..] => {
            Some(FileExtension::Audio(AudioType::Flac))
        }

        // ogg / opus
        [0x4F, 0x67, 0x67, 0x53, ..] => {
            if bytes.windows(8).any(|w| w == b"OpusHead") {
                Some(FileExtension::Audio(AudioType::Opus))
            } else {
                Some(FileExtension::Audio(AudioType::Ogg))
            }
        }

        // aac
        [0xFF, 0xF1, ..]
        | [0xFF, 0xF9, ..] => {
            Some(FileExtension::Audio(AudioType::Aac))
        }

        _ => None,
    }
}
