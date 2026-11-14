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
                bg_main: "#140E1CFF".into(),
                rubberband: "#8C4BF7".into(),
                text_primary: "#FFFFFF".into(),
                text_secondary: "#E2D5ED".into(),
                error: "#E05A5A".into(),
                warn: "#E0D65AFF".into(),
                success: "#72E05AFF".into(),
            },
            file_icons: FileTheme {
                folder_default: "#DCDC44FF".into(),
                image: "#44BADCFF".into(),
                pdf: "#DC4444FF".into(),
                document: "#445EDCFF".into(),
                video: "#B644DCFF".into(),
                audio: "#DCB044FF".into(),
                archive: "#DC9344FF".into(),
                code: "#44DC69FF".into(),
                font: "#C8C8C8FF".into(),
                executable: "#DC4444FF".into(),
                fallback: "#E0E0E0FF".into(),
            },
            sidebar_left: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#00000001".into(),
                bg_hover: "#372942".into(),
                bg_active: "#574069".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#00000000".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#2F2144FF".into(),
                bg_hover: "#372942FF".into(),
                bg_active: "#8C4BF7".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3FF".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#8080FF02".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#2F2144FF".into(),
                bg_hover: "#372942FF".into(),
                bg_active: "#6033ABFF".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3FF".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#B375F8FF".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#2F2144FF".into(),
                bg_hover: "#372942FF".into(),
                bg_active: "#6033ABFF".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3FF".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#B375F8FF".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#2F2144FF".into(),
                bg_hover: "#372942FF".into(),
                bg_active: "#6033ABFF".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3FF".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#B375F8FF".into(),
            },
            dialogs: SectionTheme {
                bg_panel: "#1C1429FF".into(),
                bg_elevated: "#2F2144FF".into(),
                bg_hover: "#372942FF".into(),
                bg_active: "#6033ABFF".into(),
                icon_primary: "#D5B5F1FF".into(),
                icon_secondary: "#9482A3FF".into(),
                accent: "#B375F8FF".into(),
                panel_border: "#B375F8FF".into(),
                component_border: "#B375F8FF".into(),
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
                bg_main: "#FCEFF3FF".into(),
                rubberband: "#E84A7AFF".into(),
                text_primary: "#29141BFF".into(),
                text_secondary: "#6E4A56FF".into(),
                error: "#D32F2FFF".into(),
                warn: "#D99B16FF".into(),
                success: "#2E7D32FF".into(),
            },
            file_icons: FileTheme::blaze_light(),
            sidebar_left: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#ECC3D0FF".into(),
                bg_active: "#DF9FB4FF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#00000000".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#EFC8D4FF".into(),
                bg_hover: "#E6B3C3FF".into(),
                bg_active: "#E84A7AFF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#00000000".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#EFC8D4FF".into(),
                bg_hover: "#E6B3C3FF".into(),
                bg_active: "#D98FA7FF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#E04878FF".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#EFC8D4FF".into(),
                bg_hover: "#E6B3C3FF".into(),
                bg_active: "#D98FA7FF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#E04878FF".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#EFC8D4FF".into(),
                bg_hover: "#E6B3C3FF".into(),
                bg_active: "#D98FA7FF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#E04878FF".into(),
            },
            dialogs: SectionTheme {
                bg_panel: "#F7DDE5FF".into(),
                bg_elevated: "#EFC8D4FF".into(),
                bg_hover: "#E6B3C3FF".into(),
                bg_active: "#D98FA7FF".into(),
                icon_primary: "#5C2839FF".into(),
                icon_secondary: "#8C6572FF".into(),
                accent: "#E04878FF".into(),
                panel_border: "#E04878FF".into(),
                component_border: "#E04878FF".into(),
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
                bg_main: "#181818FF".into(),
                rubberband: "#007ACCFF".into(),
                text_primary: "#CCCCCCFF".into(),
                text_secondary: "#9D9D9DFF".into(),
                error: "#F14C4CFF".into(),
                warn: "#CCA700FF".into(),
                success: "#89D185FF".into(),
            },
            file_icons: FileTheme::default(),
            sidebar_left: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#2A2D2EFF".into(),
                bg_active: "#37373DFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#00000000".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#2A2A2AFF".into(),
                bg_hover: "#37373DFF".into(),
                bg_active: "#007ACCFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#00000000".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#2A2A2AFF".into(),
                bg_hover: "#313335FF".into(),
                bg_active: "#04395EFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#3C3C3CFF".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#2A2A2AFF".into(),
                bg_hover: "#313335FF".into(),
                bg_active: "#04395EFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#3C3C3CFF".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#2A2A2AFF".into(),
                bg_hover: "#313335FF".into(),
                bg_active: "#04395EFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#3C3C3CFF".into(),
            },
            dialogs: SectionTheme {
                bg_panel: "#1F1F1FFF".into(),
                bg_elevated: "#2A2A2AFF".into(),
                bg_hover: "#313335FF".into(),
                bg_active: "#04395EFF".into(),
                icon_primary: "#CCCCCCFF".into(),
                icon_secondary: "#858585FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#2B2B2BFF".into(),
                component_border: "#3C3C3CFF".into(),
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
                bg_main: "#F3F3F3FF".into(),
                rubberband: "#007ACCFF".into(),
                text_primary: "#333333FF".into(),
                text_secondary: "#616161FF".into(),
                error: "#E51400FF".into(),
                warn: "#BF8803FF".into(),
                success: "#008A00FF".into(),
            },
            file_icons: FileTheme::default_light(),
            sidebar_left: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#00000000".into(),
                bg_hover: "#E8E8E8FF".into(),
                bg_active: "#D6EBFFFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#00000000".into(),
            },
            top_bar: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#F0F0F0FF".into(),
                bg_hover: "#E2E2E2FF".into(),
                bg_active: "#007ACCFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#00000000".into(),
            },
            main_area: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#F0F0F0FF".into(),
                bg_hover: "#E4E6F1FF".into(),
                bg_active: "#CCE5FFFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#D4D4D4FF".into(),
            },
            sidebar_right: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#F0F0F0FF".into(),
                bg_hover: "#E4E6F1FF".into(),
                bg_active: "#CCE5FFFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#D4D4D4FF".into(),
            },
            context_menu: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#F0F0F0FF".into(),
                bg_hover: "#E4E6F1FF".into(),
                bg_active: "#CCE5FFFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#D4D4D4FF".into(),
            },
            dialogs: SectionTheme {
                bg_panel: "#FFFFFFFF".into(),
                bg_elevated: "#F0F0F0FF".into(),
                bg_hover: "#E4E6F1FF".into(),
                bg_active: "#CCE5FFFF".into(),
                icon_primary: "#333333FF".into(),
                icon_secondary: "#767676FF".into(),
                accent: "#007ACCFF".into(),
                panel_border: "#E4E4E4FF".into(),
                component_border: "#D4D4D4FF".into(),
            },
        }
    }
}
