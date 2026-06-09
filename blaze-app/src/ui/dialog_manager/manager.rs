use std::{path::Path, sync::Arc};

use egui::{Area, Order, Sense, Ui};
use file_id::FileId;
use uuid::Uuid;

use crate::{
    core::{
        runtime::bus_structs::QuickTagEvent,
        system::fileopener_module::{platform::linux::structs::AppsIconData, AppAssociation},
    },
    ui::{
        dialog_manager::dialogs::{
            configs_dialog::ConfigDialog, error_dialog::ErrorDialog,
            folder_color_selector_dialog::FolderColorSelector,
            image_preview_dialog::ImagePreviewDialog, quick_dialogs::QuickAccDialog,
            selector_dialog::AppSelectorDialog, show_generic_message::ShowGenericDialog,
            sure_to_delete::SureToDeleteDialog, sure_to_move_to::SureToMoveToDialog,
            update_dialog::UpdateDialog, want_to_install::WantToInstallDialog,
        },
        image_preview::image_preview_handler::ImagePreviewState,
    },
};

pub trait ModalDialog {
    fn is_open(&self) -> bool;
    fn close(&mut self);
    fn render(&mut self, ui: &mut Ui) -> bool;
}

pub struct DialogManager {
    pub selector_dialog: AppSelectorDialog,
    pub sure_to_dialog: SureToMoveToDialog,
    pub update_dialog: UpdateDialog,
    pub error_dialog: ErrorDialog,
    pub sure_to_delete_dialog: SureToDeleteDialog,
    pub folder_color_dialog: FolderColorSelector,
    pub config_dialog: ConfigDialog,
    pub img_pvw_dialog: ImagePreviewDialog,
    pub want_to_install_dialog: WantToInstallDialog,
    pub generic_dialog: ShowGenericDialog,
    pub quick_dialog: QuickAccDialog,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            selector_dialog: AppSelectorDialog::new(),
            sure_to_dialog: SureToMoveToDialog::new(),
            update_dialog: UpdateDialog::new(),
            error_dialog: ErrorDialog::new(),
            sure_to_delete_dialog: SureToDeleteDialog::new(),
            folder_color_dialog: FolderColorSelector::new(),
            config_dialog: ConfigDialog::new(),
            img_pvw_dialog: ImagePreviewDialog::new(),
            want_to_install_dialog: WantToInstallDialog::new(),
            generic_dialog: ShowGenericDialog::new(),
            quick_dialog: QuickAccDialog::new(),
        }
    }

    pub fn open_selector_dialog(
        &mut self,
        path: Arc<Path>,
        mime: String,
        apps: Vec<AppAssociation>,
        icon_data: Vec<AppsIconData>,
        show_all_apps: bool,
    ) {
        self.selector_dialog
            .open(path, mime, apps, icon_data, show_all_apps);
    }

    pub fn open_sure_move_dialog(&mut self, sources: Vec<Arc<Path>>, dest: Arc<Path>) {
        self.sure_to_dialog.open(sources, dest);
    }

    pub fn open_sure_to_delete(&mut self, sources: Vec<Arc<Path>>, tab_id: Uuid) {
        self.sure_to_delete_dialog.open(sources, tab_id);
    }

    pub fn open_updater_dialog(
        &mut self,
        current_version: String,
        new_version: String,
        tab_id: Uuid,
    ) {
        self.update_dialog
            .open(current_version, new_version, tab_id);
    }

    pub fn open_error_dialog(&mut self, message: &str) {
        self.error_dialog.open(message);
    }

    pub fn open_folder_color_selector_dialog(&mut self, folder_id: FileId) {
        self.folder_color_dialog.open(folder_id);
    }

    pub fn open_configs(&mut self) {
        self.config_dialog.open();
    }

    pub fn open_img_pvw_dialog(&mut self, imp_pvw: ImagePreviewState) {
        self.img_pvw_dialog.open(imp_pvw);
    }

    pub fn open_want_to_install_dialog(&mut self) {
        self.want_to_install_dialog.open();
    }

    pub fn open_show_generic(&mut self, title: &str, message: &str) {
        self.generic_dialog.open(title, message);
    }

    pub fn open_quick_acc_dialog(&mut self, event: QuickTagEvent) {
        self.quick_dialog.open(event);
    }

    pub fn render_area(&mut self, ui: &mut Ui) {
        let dialogs: Vec<&mut dyn ModalDialog> = vec![
            &mut self.selector_dialog,
            &mut self.sure_to_dialog,
            &mut self.update_dialog,
            &mut self.error_dialog,
            &mut self.sure_to_delete_dialog,
            &mut self.folder_color_dialog,
            &mut self.config_dialog,
            &mut self.img_pvw_dialog,
            &mut self.want_to_install_dialog,
            &mut self.generic_dialog,
            &mut self.quick_dialog,
        ];

        let open_dialog = dialogs.into_iter().find(|d| d.is_open());

        if let Some(dialog) = open_dialog {
            let backdrop_clicked = {
                let screen_rect = ui.content_rect();
                let resp = Area::new("blocker".into())
                    .fixed_pos(egui::pos2(0.0, 0.0))
                    .order(Order::Middle)
                    .sense(Sense::click())
                    .interactable(true)
                    .show(ui, |ui| {
                        ui.painter().rect_filled(
                            screen_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                        );
                        ui.allocate_rect(screen_rect, egui::Sense::click())
                            .clicked()
                    });

                resp.inner
            };

            let dialog_requested_close = dialog.render(ui);

            if backdrop_clicked || dialog_requested_close {
                dialog.close();
            }
        }
    }
}
