#[allow(deprecated)]
use crate::ui::themes::platform::structs::{
    ButtonTokens, ComponentTokens, FileTheme, InputTokens, ListItemTokens, NewTheme, PanelTokens,
    SemanticTokens, SidebarItemTokens, Theme, ThemeDefault, ToolsViewTokens,
};

impl Default for FileTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}

impl FileTheme {
    pub fn default_dark() -> Self {
        Self {
            folder_default: "#FFFF00FF".into(),
            image: "#64C8FFFF".into(),
            pdf: "#FF5050FF".into(),
            document: "#648CFFFF".into(),
            video: "#C864FFFF".into(),
            audio: "#FFC850FF".into(),
            archive: "#FFA03CFF".into(),
            code: "#64FF96FF".into(),
            font: "#C8C8C8FF".into(),
            executable: "#FF6464FF".into(),
            fallback: "#FFFFFFFF".into(),
        }
    }

    pub fn default_light() -> Self {
        Self {
            folder_default: "#C49A45".into(),
            image: "#0066CC".into(),
            pdf: "#D32F2F".into(),
            document: "#1A56DB".into(),
            video: "#7B1FA2".into(),
            audio: "#E65100".into(),
            archive: "#A0522D".into(),
            code: "#007F3E".into(),
            font: "#4A4A4A".into(),
            executable: "#C62828".into(),
            fallback: "#2D2D2D".into(),
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            folder_default: "#FFA3B6".into(),
            image: "#4EA8DE".into(),
            pdf: "#E63946".into(),
            document: "#48CAE4".into(),
            video: "#B5179E".into(),
            audio: "#FFB703".into(),
            archive: "#FB8500".into(),
            code: "#2A9D8F".into(),
            font: "#6D6875".into(),
            executable: "#E63946".into(),
            fallback: "#4A4E69".into(),
        }
    }
}

impl ThemeDefault for NewTheme {
    fn default_dark() -> Self {
        Self::blaze_dark()
    }

    fn default_light() -> Self {
        Self::blaze_light()
    }

    fn default_vscode_dark() -> Self {
        Self::blaze_vscode_dark()
    }

    fn default_vscode_light() -> Self {
        Self::blaze_vscode_light()
    }
}

impl NewTheme {
    pub fn blaze_dark() -> Self {
        Self {
            name: "Blaze Dark".into(),
            autor: "Jhanfer".into(),
            version: "2.0.0".into(),
            luminance: 0.7,
            semantic: SemanticTokens {
                bg_main: "#0D0614".into(),
                bg_container: "#251731".into(),
                separator: "#574069".into(),
                accent: "#8C4BF7".into(),
                accent_glow: "#BA6EFF".into(),
                rubberband: "#8C4BF73E".into(),
                text_primary: "#FFFFFF".into(),
                text_secondary: "#E2D5ED".into(),
                text_muted: "#7A6A85".into(),
                error: "#C44D4D".into(),
                success: "#73C44D".into(),
                warn: "#C4BC4D".into(),
            },
            components: ComponentTokens {
                panel: PanelTokens {
                    bg: "#1B1124".into(),
                    border: "#BA6EFF".into(),
                },
                input: InputTokens {
                    bg: "#251731".into(),
                    border_idle: "#564663".into(),
                    border_focus: "#D3B7FF".into(),
                },
                button: ButtonTokens {
                    bg: "#281E30".into(),
                    bg_hover: "#B175DF".into(),
                    label_hover: "#D9AEFF".into(),
                    label_active: "#B45FFF".into(),
                    label_inactive: "#858585".into(),
                },
                sidebar_item: SidebarItemTokens {
                    bg_hover: "#372942".into(),
                },
                list_item: ListItemTokens {
                    bg_hover: "#372942".into(),
                    bg_selected: "#75479C".into(),
                },
                tools: ToolsViewTokens {
                    bg: "#2C1E38".into(),
                    label_hover: "#EBDCFA".into(),
                    label_active: "#9677B1".into(),
                    label_inactive: "#646464".into(),
                },
            },
            file_theme: FileTheme::default(),
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            name: "Blaze Light".into(),
            autor: "Jhanfer".into(),
            version: "2.0.0".into(),
            luminance: 0.30,
            semantic: SemanticTokens {
                bg_main: "#FFF5F7".into(),
                bg_container: "#FFE3E9".into(),
                separator: "#FFD1DC".into(),
                accent: "#FF6584".into(),
                accent_glow: "#FF85A2".into(),
                rubberband: "#FF658426".into(),
                text_primary: "#3A1A22".into(),
                text_secondary: "#704852".into(),
                text_muted: "#A3858D".into(),
                error: "#D32F2F".into(),
                success: "#388E3C".into(),
                warn: "#FBC02D".into(),
            },
            components: ComponentTokens {
                panel: PanelTokens {
                    bg: "#FFFFFF".into(),
                    border: "#FFD1DC".into(),
                },
                input: InputTokens {
                    bg: "#FFFFFF".into(),
                    border_idle: "#FFD1DC".into(),
                    border_focus: "#FF6584".into(),
                },
                button: ButtonTokens {
                    bg: "#FFE9EE".into(),
                    bg_hover: "#FFC2D1".into(),
                    label_hover: "#3A1A22".into(),
                    label_active: "#FF6584".into(),
                    label_inactive: "#A3858D".into(),
                },
                sidebar_item: SidebarItemTokens {
                    bg_hover: "#FFC2D1".into(),
                },
                list_item: ListItemTokens {
                    bg_hover: "#FFC2D1".into(),
                    bg_selected: "#E4B4BD".into(),
                },
                tools: ToolsViewTokens {
                    bg: "#FFFFFF".into(),
                    label_hover: "#FF6584".into(),
                    label_active: "#FF6584".into(),
                    label_inactive: "#A3858D".into(),
                },
            },
            file_theme: FileTheme::blaze_light(),
        }
    }

    pub fn blaze_vscode_dark() -> Self {
        Self {
            name: "VS Code Dark".into(),
            autor: "Jhanfer".into(),
            version: "2.0.0".into(),
            luminance: 0.70,
            semantic: SemanticTokens {
                bg_main: "#1E1E1E".into(),
                bg_container: "#3C3C3C".into(),
                separator: "#3C3C3C".into(),
                accent: "#007ACC".into(),
                accent_glow: "#1C97EA".into(),
                rubberband: "#007ACC33".into(),
                text_primary: "#CCCCCC".into(),
                text_secondary: "#858585".into(),
                text_muted: "#6A6A6A".into(),
                error: "#C44D4D".into(),
                success: "#73C44D".into(),
                warn: "#C4BC4D".into(),
            },
            components: ComponentTokens {
                panel: PanelTokens {
                    bg: "#252526".into(),
                    border: "#3C3C3C".into(),
                },
                input: InputTokens {
                    bg: "#3C3C3C".into(),
                    border_idle: "#3C3C3C".into(),
                    border_focus: "#007ACC".into(),
                },
                button: ButtonTokens {
                    bg: "#2D2D2D".into(),
                    bg_hover: "#2A2D2E".into(),
                    label_hover: "#FFFFFF".into(),
                    label_active: "#007ACC".into(),
                    label_inactive: "#858585".into(),
                },
                sidebar_item: SidebarItemTokens {
                    bg_hover: "#2A2D2E".into(),
                },
                list_item: ListItemTokens {
                    bg_hover: "#2A2D2E".into(),
                    bg_selected: "#5E5E69".into(),
                },
                tools: ToolsViewTokens {
                    bg: "#252526".into(),
                    label_hover: "#E5B567".into(),
                    label_active: "#E5B567".into(),
                    label_inactive: "#858585".into(),
                },
            },
            file_theme: FileTheme::default(),
        }
    }

    pub fn blaze_vscode_light() -> Self {
        Self {
            name: "VS Code Light".into(),
            autor: "Jhanfer".into(),
            version: "2.0.0".into(),
            luminance: 0.30,
            semantic: SemanticTokens {
                bg_main: "#F3F3F3".into(),
                bg_container: "#E4E4E4".into(),
                separator: "#E4E4E4".into(),
                accent: "#007ACC".into(),
                accent_glow: "#0062A3".into(),
                rubberband: "#007ACC26".into(),
                text_primary: "#555555".into(),
                text_secondary: "#636363".into(),
                text_muted: "#969696".into(),
                error: "#C44D4D".into(),
                success: "#73C44D".into(),
                warn: "#C4BC4D".into(),
            },
            components: ComponentTokens {
                panel: PanelTokens {
                    bg: "#FFFFFF".into(),
                    border: "#E4E4E4".into(),
                },
                input: InputTokens {
                    bg: "#FFFFFF".into(),
                    border_idle: "#535353".into(),
                    border_focus: "#007ACC".into(),
                },
                button: ButtonTokens {
                    bg: "#F3F3F3".into(),
                    bg_hover: "#E4E6F1".into(),
                    label_hover: "#007ACC".into(),
                    label_active: "#007ACC".into(),
                    label_inactive: "#969696".into(),
                },
                sidebar_item: SidebarItemTokens {
                    bg_hover: "#E4E6F1".into(),
                },
                list_item: ListItemTokens {
                    bg_hover: "#E4E6F1".into(),
                    bg_selected: "#8D8D8D".into(),
                },
                tools: ToolsViewTokens {
                    bg: "#FFFFFF".into(),
                    label_hover: "#007ACC".into(),
                    label_active: "#007ACC".into(),
                    label_inactive: "#858585".into(),
                },
            },
            file_theme: FileTheme::default_light(),
        }
    }
}

#[allow(deprecated)]
//migración de temas antiguos al nuevo sistema. El 'Theme' está deprecated
impl Theme {
    pub fn migrate_to_new(self) -> NewTheme {
        NewTheme {
            name: self.name,
            autor: self.autor,
            version: "2.0.0".into(),
            luminance: self.luminance,
            semantic: SemanticTokens {
                bg_main: self.bg_main,
                bg_container: self.bg_container.clone(),
                separator: self.border_panel.clone(),
                accent: self.accent.clone(),
                accent_glow: self.accent_glow,
                rubberband: self.rubberband.clone(),
                text_primary: self.text_primary.clone(),
                text_secondary: self.text_secondary.clone(),
                text_muted: self.text_muted.clone(),
                error: self.error,
                success: self.success,
                warn: self.warn,
            },
            components: ComponentTokens {
                panel: PanelTokens {
                    bg: self.bg_panel.clone(),
                    border: self.border_panel.clone(),
                },
                input: InputTokens {
                    bg: self.bg_container,
                    border_idle: self.border_panel,
                    border_focus: self.accent,
                },
                button: ButtonTokens {
                    bg: self.main_buttons,
                    bg_hover: self.bg_hover.clone(),
                    label_hover: self.tool_btn_hovered.clone(),
                    label_active: self.tool_btn_active.clone(),
                    label_inactive: self.tool_btn_inactive.clone(),
                },
                sidebar_item: SidebarItemTokens {
                    bg_hover: self.bg_hover.clone(),
                },
                list_item: ListItemTokens {
                    bg_hover: self.bg_hover,
                    bg_selected: self.item_selected,
                },
                tools: ToolsViewTokens {
                    bg: self.bg_panel,
                    label_hover: self.tool_btn_hovered,
                    label_active: self.tool_btn_active,
                    label_inactive: self.tool_btn_inactive,
                },
            },
            file_theme: self.file_theme,
        }
    }
}
