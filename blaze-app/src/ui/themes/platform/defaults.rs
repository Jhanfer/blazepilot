use crate::ui::themes::platform::structs::{
    BlazeAppTheme, FileTheme, GlobalTokens, SectionTheme, ThemeDefault, ThemeMeta,
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
            fallback: "#E0E0E0FF".into(),
        }
    }

    pub fn default_light() -> Self {
        Self {
            folder_default: "#A67D28".into(),
            image: "#0055A4".into(),
            pdf: "#B71C1C".into(),
            document: "#1544B0".into(),
            video: "#6A1B9A".into(),
            audio: "#BF360C".into(),
            archive: "#8D4018".into(),
            code: "#006430".into(),
            font: "#4A4A4A".into(),
            executable: "#B71C1C".into(),
            fallback: "#2D2D2D".into(),
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            folder_default: "#E85D7B".into(),
            image: "#2C8CBB".into(),
            pdf: "#D92635".into(),
            document: "#2C9EBB".into(),
            video: "#961382".into(),
            audio: "#E59400".into(),
            archive: "#D96A00".into(),
            code: "#1F7A6E".into(),
            font: "#5A5661".into(),
            executable: "#D92635".into(),
            fallback: "#4A4E69".into(),
        }
    }
}

impl ThemeDefault for BlazeAppTheme {
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

impl BlazeAppTheme {
    pub fn blaze_dark() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Blaze Dark".into(),
                autor: "Jhanfer".into(),
                version: "3.0.0".into(),
                luminance: 0.7,
            },
            global: GlobalTokens {
                panel_stroke_width: 0.8,
                components_stroke_width: 0.8,
                bg_main: "#0D0614".into(),
                rubberband: "#8C4BF7".into(),
                text_primary: "#FFFFFF".into(),
                text_secondary: "#E2D5ED".into(),
                error: "#E05A5A".into(),
                warn: "#D6CE5C".into(),
                success: "#85D65C".into(),
            },
            file_icons: FileTheme::default(),
            sidebar_left: SectionTheme {
                bg_panel: "#251731".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#372942".into(),
                bg_active: "#574069".into(),
                icon_primary: "#BA6EFF".into(),
                icon_secondary: "#9482A3".into(),
                accent: "#8C4BF7".into(),
                border: "#BA6EFF".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#251731".into(),
                bg_elevated: "#251731".into(),
                bg_hover: "#432E52".into(),
                bg_active: "#8C4BF7".into(),
                icon_primary: "#FFFFFF".into(),
                icon_secondary: "#858585".into(),
                accent: "#8C4BF7".into(),
                border: "#BA6EFF".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#0D0614".into(),
                bg_elevated: "#251731".into(),
                bg_hover: "#372942".into(),
                bg_active: "#75479C".into(),
                icon_primary: "#FFFFFF".into(),
                icon_secondary: "#E2D5ED".into(),
                accent: "#8C4BF7".into(),
                border: "#574069".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#2C1E38".into(),
                bg_elevated: "#251731".into(),
                bg_hover: "#432E52".into(),
                bg_active: "#8C4BF7".into(),
                icon_primary: "#FFFFFF".into(),
                icon_secondary: "#9482A3".into(),
                accent: "#8C4BF7".into(),
                border: "#BA6EFF".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#0D0614".into(),
                bg_elevated: "#251731".into(),
                bg_hover: "#372942".into(),
                bg_active: "#75479C".into(),
                icon_primary: "#FFFFFF".into(),
                icon_secondary: "#E2D5ED".into(),
                accent: "#8C4BF7".into(),
                border: "#574069".into(),
            },
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Blaze Light".into(),
                autor: "Jhanfer".into(),
                version: "3.0.0".into(),
                luminance: 0.30,
            },
            global: GlobalTokens {
                panel_stroke_width: 0.8,
                components_stroke_width: 0.8,
                bg_main: "#FFF5F7".into(),
                rubberband: "#E84A6C".into(),
                text_primary: "#291218".into(),
                text_secondary: "#5E3B44".into(),
                error: "#D32F2F".into(),
                warn: "#F9A825".into(),
                success: "#2E7D32".into(),
            },
            file_icons: FileTheme::blaze_light(),
            sidebar_left: SectionTheme {
                bg_panel: "#FFE3E9".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#FFC2D1".into(),
                bg_active: "#FFE9EE".into(),
                icon_primary: "#E84A6C".into(),
                icon_secondary: "#8C6C74".into(),
                accent: "#E84A6C".into(),
                border: "#FFD1DC".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#FFE3E9".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#FFC2D1".into(),
                bg_active: "#E84A6C".into(),
                icon_primary: "#291218".into(),
                icon_secondary: "#A3858D".into(),
                accent: "#E84A6C".into(),
                border: "#FFD1DC".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#FFF5F7".into(),
                bg_elevated: "#FFE3E9".into(),
                bg_hover: "#FFC2D1".into(),
                bg_active: "#E4B4BD".into(),
                icon_primary: "#291218".into(),
                icon_secondary: "#5E3B44".into(),
                accent: "#E84A6C".into(),
                border: "#FFD1DC".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#FFFFFF".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#FFC2D1".into(),
                bg_active: "#E84A6C".into(),
                icon_primary: "#291218".into(),
                icon_secondary: "#8C6C74".into(),
                accent: "#E84A6C".into(),
                border: "#FFD1DC".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#FFF5F7".into(),
                bg_elevated: "#FFE3E9".into(),
                bg_hover: "#FFC2D1".into(),
                bg_active: "#E4B4BD".into(),
                icon_primary: "#291218".into(),
                icon_secondary: "#5E3B44".into(),
                accent: "#E84A6C".into(),
                border: "#FFD1DC".into(),
            },
        }
    }

    pub fn blaze_vscode_dark() -> Self {
        Self {
            meta: ThemeMeta {
                name: "VS Code Dark".into(),
                autor: "Jhanfer".into(),
                version: "3.0.0".into(),
                luminance: 0.70,
            },
            global: GlobalTokens {
                panel_stroke_width: 0.8,
                components_stroke_width: 0.8,
                bg_main: "#1E1E1E".into(),
                rubberband: "#007ACC".into(),
                text_primary: "#D4D4D4".into(),
                text_secondary: "#A6A6A6".into(),
                error: "#F14C4C".into(),
                warn: "#CCA700".into(),
                success: "#89D185".into(),
            },
            file_icons: FileTheme::default(),
            sidebar_left: SectionTheme {
                bg_panel: "#2D2D2D".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#2A2D2E".into(),
                bg_active: "#37373D".into(),
                icon_primary: "#007ACC".into(),
                icon_secondary: "#808080".into(),
                accent: "#007ACC".into(),
                border: "#3C3C3C".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#2D2D2D".into(),
                bg_elevated: "#3C3C3C".into(),
                bg_hover: "#4D5054".into(),
                bg_active: "#007ACC".into(),
                icon_primary: "#D4D4D4".into(),
                icon_secondary: "#808080".into(),
                accent: "#007ACC".into(),
                border: "#3C3C3C".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#1E1E1E".into(),
                bg_elevated: "#2D2D2D".into(),
                bg_hover: "#2A2D2E".into(),
                bg_active: "#04395E".into(),
                icon_primary: "#D4D4D4".into(),
                icon_secondary: "#A6A6A6".into(),
                accent: "#007ACC".into(),
                border: "#454545".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#252526".into(),
                bg_elevated: "#3C3C3C".into(),
                bg_hover: "#4D5054".into(),
                bg_active: "#007ACC".into(),
                icon_primary: "#D4D4D4".into(),
                icon_secondary: "#808080".into(),
                accent: "#007ACC".into(),
                border: "#3C3C3C".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#1E1E1E".into(),
                bg_elevated: "#2D2D2D".into(),
                bg_hover: "#2A2D2E".into(),
                bg_active: "#04395E".into(),
                icon_primary: "#D4D4D4".into(),
                icon_secondary: "#A6A6A6".into(),
                accent: "#007ACC".into(),
                border: "#454545".into(),
            },
        }
    }

    pub fn blaze_vscode_light() -> Self {
        Self {
            meta: ThemeMeta {
                name: "VS Code Light".into(),
                autor: "Jhanfer".into(),
                version: "3.0.0".into(),
                luminance: 0.30,
            },
            global: GlobalTokens {
                panel_stroke_width: 0.8,
                components_stroke_width: 0.8,
                bg_main: "#F3F3F3".into(),
                rubberband: "#007ACC".into(),
                text_primary: "#333333".into(),
                text_secondary: "#616161".into(),
                error: "#E51400".into(),
                warn: "#BF8803".into(),
                success: "#008A00".into(),
            },
            file_icons: FileTheme::default_light(),
            sidebar_left: SectionTheme {
                bg_panel: "#FFFFFF".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#E8E8E8".into(),
                bg_active: "#007ACC".into(),
                icon_primary: "#007ACC".into(),
                icon_secondary: "#767676".into(),
                accent: "#007ACC".into(),
                border: "#E4E4E4".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#FFFFFF".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#D2D2D2".into(),
                bg_active: "#007ACC".into(),
                icon_primary: "#333333".into(),
                icon_secondary: "#767676".into(),
                accent: "#007ACC".into(),
                border: "#E4E4E4".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#F3F3F3".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#E4E6F1".into(),
                bg_active: "#007ACC20".into(),
                icon_primary: "#333333".into(),
                icon_secondary: "#616161".into(),
                accent: "#007ACC".into(),
                border: "#E4E4E4".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#FFFFFF".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#D2D2D2".into(),
                bg_active: "#007ACC".into(),
                icon_primary: "#333333".into(),
                icon_secondary: "#767676".into(),
                accent: "#007ACC".into(),
                border: "#E4E4E4".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#F3F3F3".into(),
                bg_elevated: "#FFFFFF".into(),
                bg_hover: "#E4E6F1".into(),
                bg_active: "#007ACC20".into(),
                icon_primary: "#333333".into(),
                icon_secondary: "#616161".into(),
                accent: "#007ACC".into(),
                border: "#E4E4E4".into(),
            },
        }
    }
}
