use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use eframe::egui;

pub struct FirmwareTool {
    tftp_ip: String,
    filename: String,
    device_ip: String,
    logs: String,
}

impl Default for FirmwareTool {
    fn default() -> Self {
        Self {
            tftp_ip: "192.168.1.100".to_string(),
            filename: "c2960-lanbasek9-mz.150-2.SE8.bin".to_string(),
            device_ip: "192.168.1.10".to_string(),
            logs: String::new(),
        }
    }
}

impl FirmwareTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ToolScreen for FirmwareTool {
    fn id(&self) -> &'static str {
        "firmware"
    }

    fn icon(&self) -> &'static str {
        "IOS"
    }

    fn name(&self, dil: Language) -> &'static str {
        text(dil, "Firmware (IOS) Update", "Firmware (IOS) Guncelleme")
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(text(
            dil,
            "Firmware (IOS) Update Manager",
            "Firmware (IOS) Guncelleme Yoneticisi",
        ));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(text(dil, "TFTP server IP:", "TFTP sunucu IP:")).strong());
            ui.add(egui::TextEdit::singleline(&mut self.tftp_ip).desired_width(120.0));

            ui.add_space(10.0);

            ui.label(
                egui::RichText::new(text(dil, "IOS file name (bin):", "IOS dosya adi (bin):"))
                    .strong(),
            );
            ui.add(egui::TextEdit::singleline(&mut self.filename).desired_width(200.0));
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(text(dil, "Target device IP:", "Hedef cihaz IP:")).strong(),
            );
            ui.add(egui::TextEdit::singleline(&mut self.device_ip).desired_width(120.0));
        });

        ui.add_space(20.0);

        if ui
            .button(
                egui::RichText::new(text(
                    dil,
                    "Send and update (copy tftp: flash:)",
                    "Gonder ve guncelle (copy tftp: flash:)",
                ))
                .size(16.0),
            )
            .clicked()
        {
            self.logs.push_str(&format!(
                "\n[INFO] Connecting to target {}...\n",
                self.device_ip
            ));
            self.logs.push_str(&format!(
                "[COMMAND] copy tftp://{}/{} flash:\n",
                self.tftp_ip, self.filename
            ));
            self.logs
                .push_str("[SIMULATION] Checking device flash storage...\n");
            self.logs.push_str(
                "[SIMULATION] File transfer started... (normally this takes 10-15 minutes)\n",
            );
            self.logs
                .push_str("[SUCCESS] Firmware transfer completed. Setting boot variable...\n");
            self.logs
                .push_str(&format!("[COMMAND] boot system flash:/{}\n", self.filename));
            self.logs.push_str("[INFO] Ready for reload.\n");
        }

        ui.add_space(20.0);
        ui.label(egui::RichText::new(text(dil, "Operation logs:", "Islem loglari:")).strong());

        egui::ScrollArea::both().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.logs)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(15)
                    .interactive(false),
            );
        });

        None
    }
}
