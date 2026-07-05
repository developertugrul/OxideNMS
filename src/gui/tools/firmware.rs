use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
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
        "💾"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Firmware (IOS) Güncelleme"
    }

    fn draw(&mut self, ui: &mut egui::Ui, _dil: Language) -> Option<ToolEvent> {
        ui.heading("Otomatik Firmware (IOS) Güncelleme Yöneticisi");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("TFTP Sunucu IP:").strong());
            ui.add(egui::TextEdit::singleline(&mut self.tftp_ip).desired_width(120.0));

            ui.add_space(10.0);

            ui.label(egui::RichText::new("IOS Dosya Adı (bin):").strong());
            ui.add(egui::TextEdit::singleline(&mut self.filename).desired_width(200.0));
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Hedef Cihaz IP:").strong());
            ui.add(egui::TextEdit::singleline(&mut self.device_ip).desired_width(120.0));
        });

        ui.add_space(20.0);

        if ui
            .button(egui::RichText::new("🚀 Gönder ve Güncelle (copy tftp: flash:)").size(16.0))
            .clicked()
        {
            self.logs.push_str(&format!(
                "\n[BİLGİ] {} hedefine bağlanılıyor...\n",
                self.device_ip
            ));
            self.logs.push_str(&format!(
                "[KOMUT] copy tftp://{}/{} flash:\n",
                self.tftp_ip, self.filename
            ));
            self.logs
                .push_str("[SİMÜLASYON] Cihaz hafızası kontrol ediliyor...\n");
            self.logs.push_str(
                "[SİMÜLASYON] Dosya aktarımı başlatıldı... (Bu işlem normalde 10-15 dk sürer)\n",
            );
            self.logs.push_str(
                "[BAŞARILI] Firmware aktarımı tamamlandı. Boot variable ayarlanıyor...\n",
            );
            self.logs
                .push_str(&format!("[KOMUT] boot system flash:/{}\n", self.filename));
            self.logs.push_str("[BİLGİ] Yeniden başlatma için hazır!\n");
        }

        ui.add_space(20.0);
        ui.label(egui::RichText::new("İşlem Logları:").strong());

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
