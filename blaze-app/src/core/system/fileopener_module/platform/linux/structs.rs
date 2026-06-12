use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum OpenerFileKind {
    AppImage(u8),
    ElfExecutable,
    ShellScript,
    PythonScript,
    RubyScript,
    PerlScript,
    NodeScript,
    OtherScript,
    Png,
    Jpeg,
    Pdf,
    Zip,
    Unknown,
}

impl OpenerFileKind {
    pub fn is_directly_executable(&self) -> bool {
        matches!(
            self,
            OpenerFileKind::AppImage(_) | OpenerFileKind::ElfExecutable
        )
    }

    pub fn detect(path: &Path) -> Self {
        use std::io::Read;

        let mut f = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return OpenerFileKind::Unknown,
        };

        let mut buf = [0u8; 16];
        if f.read_exact(&mut buf).is_err() {
            return OpenerFileKind::Unknown;
        }

        match &buf {
            // AppImage tipo 2
            [0x7f, b'E', b'L', b'F', ..] if buf[8..11] == [0x41, 0x49, 0x02] => {
                OpenerFileKind::AppImage(2)
            }
            // AppImage tipo 1
            [0x7f, b'E', b'L', b'F', ..] if buf[8..11] == [0x41, 0x49, 0x01] => {
                OpenerFileKind::AppImage(1)
            }
            // ELF genérico
            [0x7f, b'E', b'L', b'F', ..] => OpenerFileKind::ElfExecutable,
            // Shebangs conocidos
            [b'#', b'!', b'/', b'b', b'i', b'n', b'/', b's', b'h', ..] => {
                OpenerFileKind::ShellScript
            }
            [b'#', b'!', b'/', b'b', b'i', b'n', b'/', b'b', b'a', b's', b'h', ..] => {
                OpenerFileKind::ShellScript
            }
            // Cualquier otro shebang, se lee la línea completa
            [b'#', b'!', ..] => Self::classify_shebang(path),
            // PNG
            [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => OpenerFileKind::Png,
            // JPEG
            [0xFF, 0xD8, 0xFF, ..] => OpenerFileKind::Jpeg,
            // PDF
            [b'%', b'P', b'D', b'F', ..] => OpenerFileKind::Pdf,
            // ZIP
            [0x50, 0x4B, 0x03, 0x04, ..] => OpenerFileKind::Zip,
            // Fallback: intentar por extensión
            _ => Self::detect_by_extension(path),
        }
    }

    fn classify_shebang(path: &Path) -> OpenerFileKind {
        let Ok(content) = std::fs::read_to_string(path) else {
            return OpenerFileKind::OtherScript;
        };
        let line = content.lines().next().unwrap_or("");

        if line.contains("python") {
            OpenerFileKind::PythonScript
        } else if line.contains("ruby") {
            OpenerFileKind::RubyScript
        } else if line.contains("perl") {
            OpenerFileKind::PerlScript
        } else if line.contains("node") || line.contains("deno") {
            OpenerFileKind::NodeScript
        } else {
            OpenerFileKind::OtherScript
        }
    }

    fn detect_by_extension(path: &Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "appimage" => return OpenerFileKind::AppImage(0),
                "png" => return OpenerFileKind::Png,
                "jpg" | "jpeg" => return OpenerFileKind::Jpeg,
                "pdf" => return OpenerFileKind::Pdf,
                "zip" => return OpenerFileKind::Zip,
                "sh" | "bash" => return OpenerFileKind::ShellScript,
                "py" => return OpenerFileKind::PythonScript,
                "rb" => return OpenerFileKind::RubyScript,
                "pl" => return OpenerFileKind::PerlScript,
                "js" => return OpenerFileKind::NodeScript,
                _ => {}
            }
        }

        OpenerFileKind::Unknown
    }
}
