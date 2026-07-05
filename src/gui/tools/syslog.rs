use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use chrono::{DateTime, Local};
use eframe::egui;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
struct SyslogMsg {
    timestamp: DateTime<Local>,
    source_ip: String,
    message: String,
    severity: String,
}

pub struct SyslogTool {
    messages: Arc<Mutex<Vec<SyslogMsg>>>,
    is_running: Arc<Mutex<bool>>,
    bind_port: u16,
    status_msg: String,
    auto_scroll: bool,
}

impl Default for SyslogTool {
    fn default() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            bind_port: 514,
            status_msg: "Stopped".to_string(),
            auto_scroll: true,
        }
    }
}

impl SyslogTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn start_server(&mut self, ctx: egui::Context) {
        if *self.is_running.lock().unwrap() {
            return;
        }

        let bind_addr = format!("0.0.0.0:{}", self.bind_port);
        let socket = match UdpSocket::bind(&bind_addr) {
            Ok(s) => s,
            Err(e) => {
                self.status_msg = format!("Port error (administrator rights may be required): {e}");
                return;
            }
        };

        if socket.set_nonblocking(true).is_err() {
            self.status_msg = "Non-blocking mode failed".to_string();
            return;
        }

        self.status_msg = format!("Listening on port {}...", self.bind_port);
        *self.is_running.lock().unwrap() = true;

        let messages_clone = self.messages.clone();
        let running_clone = self.is_running.clone();

        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                if !*running_clone.lock().unwrap() {
                    break;
                }

                match socket.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        let raw_msg = String::from_utf8_lossy(&buf[..amt]).to_string();
                        let mut severity = "INFO".to_string();
                        let mut msg_body = raw_msg.clone();

                        if raw_msg.starts_with('<')
                            && let Some(end_idx) = raw_msg.find('>')
                        {
                            let pri_val_str = &raw_msg[1..end_idx];
                            if let Ok(pri_val) = pri_val_str.parse::<u8>() {
                                let sev = pri_val & 0x07;
                                severity = match sev {
                                    0 => "EMERG".to_string(),
                                    1 => "ALERT".to_string(),
                                    2 => "CRIT".to_string(),
                                    3 => "ERR".to_string(),
                                    4 => "WARNING".to_string(),
                                    5 => "NOTICE".to_string(),
                                    6 => "INFO".to_string(),
                                    7 => "DEBUG".to_string(),
                                    _ => "UNKNOWN".to_string(),
                                };
                            }
                            msg_body = raw_msg[end_idx + 1..].trim().to_string();
                        }

                        let log_entry = SyslogMsg {
                            timestamp: Local::now(),
                            source_ip: src.ip().to_string(),
                            severity,
                            message: msg_body,
                        };

                        if let Ok(mut lock) = messages_clone.lock() {
                            lock.push(log_entry);
                            if lock.len() > 1000 {
                                lock.remove(0);
                            }
                        }
                        ctx.request_repaint();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        });
    }

    fn stop_server(&mut self) {
        if let Ok(mut lock) = self.is_running.lock() {
            *lock = false;
        }
        self.status_msg = "Stopped".to_string();
    }
}

impl ToolScreen for SyslogTool {
    fn id(&self) -> &'static str {
        "syslog"
    }

    fn icon(&self) -> &'static str {
        "SYS"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Syslog Server"
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(text(
            dil,
            "Syslog Server (Live Alerts)",
            "Syslog Sunucusu (Canli Alarmlar)",
        ));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(text(dil, "Listen port:", "Dinlenecek port:"));
            ui.add(egui::DragValue::new(&mut self.bind_port));

            let is_running = *self.is_running.lock().unwrap();

            if is_running {
                if ui.button(text(dil, "Stop", "Durdur")).clicked() {
                    self.stop_server();
                }
            } else if ui.button(text(dil, "Start", "Baslat")).clicked() {
                self.start_server(ui.ctx().clone());
            }

            ui.label(egui::RichText::new(&self.status_msg).color(if is_running {
                egui::Color32::GREEN
            } else {
                egui::Color32::GRAY
            }));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(text(dil, "Clear logs", "Loglari temizle"))
                    .clicked()
                    && let Ok(mut lock) = self.messages.lock()
                {
                    lock.clear();
                }
                ui.checkbox(
                    &mut self.auto_scroll,
                    text(dil, "Auto-scroll", "Oto-kaydir"),
                );
            });
        });

        ui.add_space(10.0);
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                let messages = self.messages.lock().unwrap().clone();

                egui::Grid::new("syslog_grid")
                    .striped(true)
                    .spacing([15.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(text(dil, "Time", "Zaman")).strong());
                        ui.label(egui::RichText::new(text(dil, "Source IP", "Kaynak IP")).strong());
                        ui.label(egui::RichText::new(text(dil, "Severity", "Seviye")).strong());
                        ui.label(egui::RichText::new(text(dil, "Message", "Mesaj")).strong());
                        ui.end_row();

                        for msg in messages {
                            ui.label(msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                            ui.label(&msg.source_ip);

                            let sev_color = match msg.severity.as_str() {
                                "EMERG" | "ALERT" | "CRIT" => egui::Color32::RED,
                                "ERR" => egui::Color32::LIGHT_RED,
                                "WARNING" => egui::Color32::YELLOW,
                                "NOTICE" => egui::Color32::LIGHT_BLUE,
                                _ => ui.visuals().text_color(),
                            };

                            ui.label(egui::RichText::new(&msg.severity).color(sev_color));
                            ui.label(&msg.message);
                            ui.end_row();
                        }
                    });
            });

        None
    }
}
