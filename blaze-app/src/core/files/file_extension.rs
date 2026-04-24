#[derive(Debug, Clone, PartialEq)]
pub enum DocType {
    Pdf,
    Doc, Docx,
    Xls, Xlsx,
    Ppt, Pptx,
    Txt, Md, Rtf, Csv, Log,
    Odt, Ods, Odp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageType {
    Png, Jpg, Gif, Webp, Bmp, Tiff, Svg, Ico, Avif, Heic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoType {
    Mp4, Mkv, Avi, Mov, Wmv, Flv, Webm, M4v,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioType {
    Mp3, Wav, Flac, Ogg, Aac, M4a, Opus, Wma,
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
    Zst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeType {
    Rs, Py, Js, Ts, C, Cpp, H, Hpp,
    Go, Java, Kt, Swift, Rb, Php,
    Html, Css, Scss, Json, Toml, Yaml, Xml,
    Sh, Bash, Fish, Zsh,
    Sql, R, Lua, Dart,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontType {
    Ttf, Otf, Woff, Woff2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableType {
    //Linux
    AppImage,
    Deb, Rpm,
    //Windows
    Exe, Msi,
    //Macos
    Dmg, App,
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


        match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
            Some("pdf")  => Self::Document(DocType::Pdf),
            Some("doc")  => Self::Document(DocType::Doc),
            Some("docx") => Self::Document(DocType::Docx),
            Some("xls")  => Self::Document(DocType::Xls),
            Some("xlsx") => Self::Document(DocType::Xlsx),
            Some("ppt")  => Self::Document(DocType::Ppt),
            Some("pptx") => Self::Document(DocType::Pptx),
            Some("txt")  => Self::Document(DocType::Txt),
            Some("md")   => Self::Document(DocType::Md),
            Some("rtf")  => Self::Document(DocType::Rtf),
            Some("csv")  => Self::Document(DocType::Csv),
            Some("log")  => Self::Document(DocType::Log),
            Some("odt")  => Self::Document(DocType::Odt),
            Some("ods")  => Self::Document(DocType::Ods),
            Some("odp")  => Self::Document(DocType::Odp),

            Some("png")  => Self::Image(ImageType::Png),
            Some("jpg") | Some("jpeg") => Self::Image(ImageType::Jpg),
            Some("gif")  => Self::Image(ImageType::Gif),
            Some("webp") => Self::Image(ImageType::Webp),
            Some("bmp")  => Self::Image(ImageType::Bmp),
            Some("tiff") | Some("tif") => Self::Image(ImageType::Tiff),
            Some("svg")  => Self::Image(ImageType::Svg),
            Some("ico")  => Self::Image(ImageType::Ico),
            Some("avif") => Self::Image(ImageType::Avif),
            Some("heic") => Self::Image(ImageType::Heic),

            Some("mp4")  => Self::Video(VideoType::Mp4),
            Some("mkv")  => Self::Video(VideoType::Mkv),
            Some("avi")  => Self::Video(VideoType::Avi),
            Some("mov")  => Self::Video(VideoType::Mov),
            Some("wmv")  => Self::Video(VideoType::Wmv),
            Some("flv")  => Self::Video(VideoType::Flv),
            Some("webm") => Self::Video(VideoType::Webm),
            Some("m4v")  => Self::Video(VideoType::M4v),

            Some("mp3")  => Self::Audio(AudioType::Mp3),
            Some("wav")  => Self::Audio(AudioType::Wav),
            Some("flac") => Self::Audio(AudioType::Flac),
            Some("ogg")  => Self::Audio(AudioType::Ogg),
            Some("aac")  => Self::Audio(AudioType::Aac),
            Some("m4a")  => Self::Audio(AudioType::M4a),
            Some("opus") => Self::Audio(AudioType::Opus),
            Some("wma")  => Self::Audio(AudioType::Wma),

            Some("zip") => Self::Archive(ArchiveType::Zip),
            Some("tar") => Self::Archive(ArchiveType::Tar),
            Some("gz")  => Self::Archive(ArchiveType::Gz),
            Some("bz2") => Self::Archive(ArchiveType::Bz2),
            Some("xz")  => Self::Archive(ArchiveType::Xz),
            Some("7z")  => Self::Archive(ArchiveType::SevenZ),
            Some("rar") => Self::Archive(ArchiveType::Rar),

            Some("rs")   => Self::Code(CodeType::Rs),
            Some("py")   => Self::Code(CodeType::Py),
            Some("js")   => Self::Code(CodeType::Js),
            Some("ts")   => Self::Code(CodeType::Ts),
            Some("c")    => Self::Code(CodeType::C),
            Some("cpp")  => Self::Code(CodeType::Cpp),
            Some("h")    => Self::Code(CodeType::H),
            Some("hpp")  => Self::Code(CodeType::Hpp),
            Some("go")   => Self::Code(CodeType::Go),
            Some("java") => Self::Code(CodeType::Java),
            Some("kt")   => Self::Code(CodeType::Kt),
            Some("swift")=> Self::Code(CodeType::Swift),
            Some("rb")   => Self::Code(CodeType::Rb),
            Some("php")  => Self::Code(CodeType::Php),
            Some("html") => Self::Code(CodeType::Html),
            Some("css")  => Self::Code(CodeType::Css),
            Some("scss") => Self::Code(CodeType::Scss),
            Some("json") => Self::Code(CodeType::Json),
            Some("toml") => Self::Code(CodeType::Toml),
            Some("yaml") | Some("yml") => Self::Code(CodeType::Yaml),
            Some("xml")  => Self::Code(CodeType::Xml),
            Some("sh")   => Self::Code(CodeType::Sh),
            Some("bash") => Self::Code(CodeType::Bash),
            Some("fish") => Self::Code(CodeType::Fish),
            Some("zsh")  => Self::Code(CodeType::Zsh),
            Some("sql")  => Self::Code(CodeType::Sql),
            Some("r")    => Self::Code(CodeType::R),
            Some("lua")  => Self::Code(CodeType::Lua),
            Some("dart") => Self::Code(CodeType::Dart),

            Some("ttf")   => Self::Font(FontType::Ttf),
            Some("otf")   => Self::Font(FontType::Otf),
            Some("woff")  => Self::Font(FontType::Woff),
            Some("woff2") => Self::Font(FontType::Woff2),

            Some("deb")      => Self::Executable(ExecutableType::Deb),
            Some("rpm")      => Self::Executable(ExecutableType::Rpm),
            Some("appimage") => Self::Executable(ExecutableType::AppImage),
            Some("exe")      => Self::Executable(ExecutableType::Exe),
            Some("msi")      => Self::Executable(ExecutableType::Msi),
            Some("dmg")      => Self::Executable(ExecutableType::Dmg),
            Some("app")      => Self::Executable(ExecutableType::App),

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

    pub fn is_code(&self) -> bool {
        matches!(self, FileExtension::Code(_))
    }

    pub fn archive_type(&self) -> Option<ArchiveType> {
        match self {
            FileExtension::Archive(t) => Some(t.clone()),
            _ => None,
        }
    }
}