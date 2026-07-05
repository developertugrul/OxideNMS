use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;
use snmp::{SyncSession, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Node {
    id: i32,
    name: String,
    ip: String,
    pos: egui::Pos2,
    is_up: bool,
    sys_descr: String,
}

pub struct SnmpMapTool {
    nodes: Arc<Mutex<Vec<Node>>>,
    is_polling: Arc<Mutex<bool>>,
    dragging_node: Option<i32>,
}

impl Default for SnmpMapTool {
    fn default() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(Vec::new())),
            is_polling: Arc::new(Mutex::new(false)),
            dragging_node: None,
        }
    }
}

impl SnmpMapTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_devices(&mut self) {
        let mut list = Vec::new();
        if let Ok(conn) = db::get_connection() {
            if let Ok(mut stmt) = conn.prepare("SELECT id, name, ip_address FROM devices") {
                if let Ok(iter) = stmt.query_map([], |row| {
                    Ok(Node {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        ip: row.get(2)?,
                        pos: egui::pos2(100.0, 100.0), // Will spread them out later
                        is_up: false,
                        sys_descr: String::new(),
                    })
                }) {
                    let mut x = 100.0;
                    let mut y = 100.0;
                    for dev_res in iter {
                        if let Ok(mut dev) = dev_res {
                            dev.pos = egui::pos2(x, y);
                            x += 150.0;
                            if x > 600.0 {
                                x = 100.0;
                                y += 150.0;
                            }
                            list.push(dev);
                        }
                    }
                }
            }
        }
        if let Ok(mut l) = self.nodes.lock() {
            *l = list;
        }
    }

    fn start_polling(&mut self, ctx: egui::Context) {
        if let Ok(mut lock) = self.is_polling.lock() {
            if *lock {
                return;
            }
            *lock = true;
        }

        let nodes_arc = self.nodes.clone();
        let is_polling = self.is_polling.clone();

        thread::spawn(move || {
            loop {
                if let Ok(lock) = is_polling.lock() {
                    if !*lock {
                        break;
                    }
                }

                let len = {
                    let lock = nodes_arc.lock().unwrap();
                    lock.len()
                };

                for i in 0..len {
                    let ip = {
                        let lock = nodes_arc.lock().unwrap();
                        lock[i].ip.clone()
                    };

                    let sys_descr_oid = &[1, 3, 6, 1, 2, 1, 1, 1, 0];
                    let addr = format!("{}:161", ip);

                    let mut is_up = false;
                    let mut descr = String::new();

                    if let Ok(mut sess) =
                        SyncSession::new(addr, b"public", Some(Duration::from_secs(2)), 0)
                    {
                        if let Ok(mut response) = sess.get(sys_descr_oid) {
                            is_up = true;
                            if let Some((_oid, Value::OctetString(sys_descr))) =
                                response.varbinds.next()
                            {
                                descr = String::from_utf8_lossy(sys_descr).into_owned();
                            }
                        }
                    }

                    if let Ok(mut lock) = nodes_arc.lock() {
                        lock[i].is_up = is_up;
                        lock[i].sys_descr = descr;
                    }
                }

                ctx.request_repaint();
                thread::sleep(Duration::from_secs(10));
            }
        });
    }

    fn stop_polling(&mut self) {
        if let Ok(mut lock) = self.is_polling.lock() {
            *lock = false;
        }
    }
}

impl ToolScreen for SnmpMapTool {
    fn id(&self) -> &'static str {
        "snmp_map"
    }

    fn icon(&self) -> &'static str {
        "🗺️"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "SNMP Map"
    }

    fn draw(&mut self, ui: &mut egui::Ui, _dil: Language) -> Option<ToolEvent> {
        ui.heading("Topoloji Haritası (SNMP)");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Cihazları Yükle").clicked() {
                self.load_devices();
            }

            let running = *self.is_polling.lock().unwrap();
            if running {
                ui.label(egui::RichText::new("SNMP Polling Aktif...").color(egui::Color32::GREEN));
                if ui.button("Durdur").clicked() {
                    self.stop_polling();
                }
            } else {
                if ui.button("İzlemeyi Başlat (SNMP)").clicked() {
                    self.start_polling(ui.ctx().clone());
                }
            }
        });

        ui.add_space(10.0);

        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let rect = response.rect;

        // Draw background
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(30, 30, 30));

        let mut nodes = self.nodes.lock().unwrap();

        // Handle dragging
        if let Some(id) = self.dragging_node {
            if response.dragged() {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    node.pos += response.drag_delta();
                }
            }
            if response.drag_stopped() {
                self.dragging_node = None;
            }
        } else if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                for node in nodes.iter() {
                    let node_rect = egui::Rect::from_center_size(
                        rect.min + node.pos.to_vec2(),
                        egui::vec2(100.0, 60.0),
                    );
                    if node_rect.contains(pos) {
                        self.dragging_node = Some(node.id);
                        break;
                    }
                }
            }
        }

        // Draw nodes
        for node in nodes.iter() {
            let center = rect.min + node.pos.to_vec2();
            let node_rect = egui::Rect::from_center_size(center, egui::vec2(120.0, 70.0));

            let fill_color = if node.is_up {
                egui::Color32::from_rgb(40, 100, 40)
            } else {
                egui::Color32::from_rgb(100, 40, 40)
            };
            let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);

            painter.rect_filled(node_rect, 8.0, fill_color);
            painter.rect_stroke(node_rect, 8.0, stroke, egui::StrokeKind::Inside);

            let text = format!(
                "{}\n{}\n{}",
                node.name,
                node.ip,
                if node.is_up { "UP" } else { "DOWN" }
            );
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        }

        None
    }
}
