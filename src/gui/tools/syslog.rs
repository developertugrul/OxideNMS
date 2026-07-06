use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use eframe::egui;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct SyslogTool {
    is_running: Arc<Mutex<bool>>,
    bind_port: u16,
    status_msg: String,
    auto_scroll: bool,
    source_filter: String,
    severity_filter: String,
    text_filter: String,
    event_limit: i32,
    export_status: String,
}

impl Default for SyslogTool {
    fn default() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            bind_port: 514,
            status_msg: "Stopped".to_string(),
            auto_scroll: true,
            source_filter: String::new(),
            severity_filter: String::new(),
            text_filter: String::new(),
            event_limit: 200,
            export_status: String::new(),
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

                        let source_ip = src.ip().to_string();

                        if let Ok(conn) = db::get_connection() {
                            let _ = db::syslog::save_event(
                                &conn, &source_ip, &severity, &msg_body, &raw_msg,
                            );
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

    fn persisted_events(&self) -> Vec<db::syslog::SyslogEvent> {
        let Ok(conn) = db::get_connection() else {
            return Vec::new();
        };

        db::syslog::search(
            &conn,
            &self.source_filter,
            &self.severity_filter,
            &self.text_filter,
            self.event_limit as i64,
        )
        .unwrap_or_default()
    }

    fn copy_csv(&mut self, ctx: &egui::Context, events: &[db::syslog::SyslogEvent], dil: Language) {
        let mut csv = String::from("id,received_at,source_ip,severity,message,raw_message\r\n");
        for event in events {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\r\n",
                event.id,
                csv_cell(&event.received_at),
                csv_cell(&event.source_ip),
                csv_cell(&event.severity),
                csv_cell(&event.message),
                csv_cell(&event.raw_message)
            ));
        }

        ctx.copy_text(csv);
        self.export_status =
            text(dil, "CSV copied to clipboard.", "CSV panoya kopyalandi.").to_string();
    }
}

fn csv_cell(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
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
                    .button(text(dil, "Clear filters", "Filtreleri temizle"))
                    .clicked()
                {
                    self.source_filter.clear();
                    self.severity_filter.clear();
                    self.text_filter.clear();
                    self.export_status.clear();
                }
                ui.checkbox(
                    &mut self.auto_scroll,
                    text(dil, "Auto-scroll", "Oto-kaydir"),
                );
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(egui::RichText::new(text(dil, "Persisted events", "Kalici olaylar")).strong());
        ui.horizontal_wrapped(|ui| {
            ui.label(text(dil, "Source:", "Kaynak:"));
            ui.add(
                egui::TextEdit::singleline(&mut self.source_filter)
                    .desired_width(130.0)
                    .hint_text("192.0.2.10"),
            );

            ui.label(text(dil, "Severity:", "Seviye:"));
            egui::ComboBox::from_id_salt("syslog_severity_filter")
                .selected_text(if self.severity_filter.is_empty() {
                    text(dil, "All", "Tumu")
                } else {
                    self.severity_filter.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.severity_filter,
                        String::new(),
                        text(dil, "All", "Tumu"),
                    );
                    for severity in [
                        "EMERG", "ALERT", "CRIT", "ERR", "WARNING", "NOTICE", "INFO", "DEBUG",
                    ] {
                        ui.selectable_value(
                            &mut self.severity_filter,
                            severity.to_string(),
                            severity,
                        );
                    }
                });

            ui.label(text(dil, "Search:", "Ara:"));
            ui.add(
                egui::TextEdit::singleline(&mut self.text_filter)
                    .desired_width(180.0)
                    .hint_text("LINK-3-UPDOWN"),
            );

            ui.label(text(dil, "Limit:", "Limit:"));
            ui.add(egui::DragValue::new(&mut self.event_limit).range(20..=5000));
        });

        let events = self.persisted_events();
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}",
                text(dil, "Loaded", "Yuklenen"),
                events.len()
            ));
            if ui.button(text(dil, "Copy CSV", "CSV kopyala")).clicked() {
                self.copy_csv(ui.ctx(), &events, dil);
            }
            if !self.export_status.is_empty() {
                ui.label(egui::RichText::new(&self.export_status).weak());
            }
        });

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                egui::Grid::new("syslog_grid")
                    .striped(true)
                    .spacing([15.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(text(dil, "Time", "Zaman")).strong());
                        ui.label(egui::RichText::new(text(dil, "Source IP", "Kaynak IP")).strong());
                        ui.label(egui::RichText::new(text(dil, "Severity", "Seviye")).strong());
                        ui.label(egui::RichText::new(text(dil, "Message", "Mesaj")).strong());
                        ui.end_row();

                        for event in &events {
                            ui.label(&event.received_at);
                            ui.label(&event.source_ip);

                            let sev_color = match event.severity.as_str() {
                                "EMERG" | "ALERT" | "CRIT" => egui::Color32::RED,
                                "ERR" => egui::Color32::LIGHT_RED,
                                "WARNING" => egui::Color32::YELLOW,
                                "NOTICE" => egui::Color32::LIGHT_BLUE,
                                _ => ui.visuals().text_color(),
                            };

                            ui.label(egui::RichText::new(&event.severity).color(sev_color));
                            ui.label(&event.message);
                            ui.end_row();
                        }
                    });
            });

        None
    }
}
