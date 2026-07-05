use eframe::egui;
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use super::{ToolEvent, ToolScreen};
use crate::db;
use crate::i18n::{Language, Message, t};

enum SshState {
    Idle,
    Connecting,
    Done(String),
    Error(String),
}

pub struct SshTool {
    ip: String,
    port: String,
    user: String,
    pass: String,
    state: SshState,
    rx: Option<Receiver<Result<String, String>>>,
    db_mesaj: Option<String>,
}

impl Default for SshTool {
    fn default() -> Self {
        Self {
            ip: "192.168.1.1".to_owned(),
            port: "22".to_owned(),
            user: "admin".to_owned(),
            pass: "cisco".to_owned(),
            state: SshState::Idle,
            rx: None,
            db_mesaj: None,
        }
    }
}

impl ToolScreen for SshTool {
    fn icon(&self) -> &'static str {
        "🔌"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::SshName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::SshName));
        ui.label(t(dil, Message::SshDescription));
        ui.add_space(10.0);

        // Check if there is a message from the background thread
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(config) => self.state = SshState::Done(config),
                    Err(e) => self.state = SshState::Error(e),
                }
                self.rx = None;
            }
        }

        ui.horizontal(|ui| {
            ui.label(t(dil, Message::SshIp));
            ui.text_edit_singleline(&mut self.ip);
            ui.label(t(dil, Message::SshPort));
            ui.text_edit_singleline(&mut self.port);
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(t(dil, Message::SshUser));
            ui.text_edit_singleline(&mut self.user);
            ui.label(t(dil, Message::SshPass));
            ui.add(egui::TextEdit::singleline(&mut self.pass).password(true));
        });

        ui.add_space(10.0);

        let is_connecting = matches!(self.state, SshState::Connecting);

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !is_connecting,
                    egui::Button::new(t(dil, Message::SshConnect)),
                )
                .clicked()
            {
                self.state = SshState::Connecting;
                self.db_mesaj = None;

                let (tx, rx) = channel();
                self.rx = Some(rx);

                let ip = self.ip.clone();
                let port = self.port.clone();
                let user = self.user.clone();
                let pass = self.pass.clone();

                let ctx = ui.ctx().clone();

                thread::spawn(move || {
                    let res = connect_and_fetch(&ip, &port, &user, &pass);
                    let _ = tx.send(res);
                    ctx.request_repaint();
                });
            }

            if is_connecting {
                ui.spinner();
                ui.label(t(dil, Message::SshConnectingLabel));
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        match &self.state {
            SshState::Idle => {}
            SshState::Connecting => {}
            SshState::Error(e) => {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 80, 80),
                    format!("{} {}", t(dil, Message::ErrorPrefix), e),
                );
            }
            SshState::Done(config) => {
                let mut return_event = None;
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t(dil, Message::SshSuccess))
                            .color(egui::Color32::from_rgb(80, 255, 80)),
                    );

                    if ui.button(t(dil, Message::DiffSaveToDb)).clicked() {
                        match db::get_connection() {
                            Ok(conn) => {
                                let device_id = match db::devices::get_or_create_device(
                                    &conn, &self.ip, &self.ip,
                                ) {
                                    Ok(id) => id,
                                    Err(_) => 1, // Fallback
                                };

                                // Kaydetmeden önce önceki konfigürasyonu alalım
                                let mut prev_config = String::new();
                                if let Ok(history) =
                                    db::devices::get_config_history(&conn, device_id)
                                {
                                    if let Some(last) = history.first() {
                                        prev_config = last.config_text.clone();
                                    }
                                }

                                match db::devices::save_config(&conn, device_id, config) {
                                    Ok(_) => {
                                        self.db_mesaj = Some("Veritabanına Kaydedildi!".to_owned());
                                        if !prev_config.is_empty() {
                                            return_event = Some(ToolEvent::SwitchToDiff {
                                                old_config: prev_config,
                                                new_config: config.clone(),
                                            });
                                        }
                                    }
                                    Err(e) => self.db_mesaj = Some(format!("Kayıt Hatası: {}", e)),
                                }
                            }
                            Err(e) => self.db_mesaj = Some(format!("DB Hatası: {}", e)),
                        }
                    }

                    if let Some(msg) = &self.db_mesaj {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::YELLOW));
                    }
                });

                if return_event.is_some() {
                    return return_event;
                }

                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut config.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .code_editor(),
                    );
                });
            }
        }

        None
    }
}

fn connect_and_fetch(ip: &str, port: &str, user: &str, pass: &str) -> Result<String, String> {
    use ssh;

    let addr = format!("{}:{}", ip, port);

    // Disable logging output to stdout if it's annoying

    let mut session = ssh::create_session()
        .username(user)
        .password(pass)
        .connect(&addr)
        .map_err(|e| format!("Bağlantı kurulamadı: {:?}", e))?
        .run_local();

    let exec = session
        .open_exec()
        .map_err(|e| format!("Exec kanalı açılamadı: {:?}", e))?;
    let vec: Vec<u8> = exec
        .send_command("show running-config")
        .map_err(|e| format!("Komut gönderilemedi: {:?}", e))?;

    let output = String::from_utf8_lossy(&vec).into_owned();

    session.close();

    Ok(output)
}
