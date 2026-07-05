use eframe::egui;

use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};

#[derive(Default)]
pub struct AuditLogTool {
    filter: String,
    rows: Vec<AuditRow>,
    status_msg: String,
}

#[derive(Clone)]
struct AuditRow {
    occurred_at: String,
    action: String,
    target: String,
    status: String,
    details: String,
}

impl AuditLogTool {
    fn refresh(&mut self) {
        self.rows.clear();
        let Ok(conn) = db::get_connection() else {
            self.status_msg = "Audit database could not be opened.".to_string();
            return;
        };

        let like = format!("%{}%", self.filter.trim());
        let query = if self.filter.trim().is_empty() {
            "SELECT occurred_at, action, target, status, COALESCE(details, '')
             FROM audit_log ORDER BY id DESC LIMIT 300"
        } else {
            "SELECT occurred_at, action, target, status, COALESCE(details, '')
             FROM audit_log
             WHERE action LIKE ?1 OR target LIKE ?1 OR status LIKE ?1 OR details LIKE ?1
             ORDER BY id DESC LIMIT 300"
        };

        let mut stmt = match conn.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => {
                self.status_msg = format!("Audit query could not be prepared: {e}");
                return;
            }
        };

        let mapped = if self.filter.trim().is_empty() {
            stmt.query_map([], map_row)
        } else {
            stmt.query_map([like], map_row)
        };

        match mapped {
            Ok(rows) => {
                self.rows.extend(rows.flatten());
                self.status_msg = format!("{} audit records listed.", self.rows.len());
            }
            Err(e) => self.status_msg = format!("Audit read failed: {e}"),
        }
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditRow> {
    Ok(AuditRow {
        occurred_at: row.get(0)?,
        action: row.get(1)?,
        target: row.get(2)?,
        status: row.get(3)?,
        details: row.get(4)?,
    })
}

impl ToolScreen for AuditLogTool {
    fn id(&self) -> &'static str {
        "audit_log"
    }

    fn icon(&self) -> &'static str {
        "AUD"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Audit Log"
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        if self.rows.is_empty() && self.status_msg.is_empty() {
            self.refresh();
        }

        ui.heading("Audit Log");
        ui.label(text(
            dil,
            "Operational trail for device, vault, discovery, backup, and bulk deploy actions.",
            "Cihaz, vault, discovery, backup ve bulk deploy operasyon izleri.",
        ));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(text(dil, "Filter", "Filtre"));
            ui.text_edit_singleline(&mut self.filter);
            if ui.button(text(dil, "Refresh", "Yenile")).clicked() {
                self.refresh();
            }
            if ui.button(text(dil, "Copy CSV", "CSV kopyala")).clicked() {
                ui.ctx().copy_text(csv_export(&self.rows));
            }
        });

        if !self.status_msg.is_empty() {
            ui.label(egui::RichText::new(&self.status_msg).small().weak());
        }

        ui.add_space(8.0);
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("audit_log_grid")
                .striped(true)
                .spacing([18.0, 8.0])
                .show(ui, |ui| {
                    for header in ["Time", "Action", "Target", "Status", "Details"] {
                        ui.label(egui::RichText::new(header).strong());
                    }
                    ui.end_row();

                    for row in &self.rows {
                        ui.label(&row.occurred_at);
                        ui.label(&row.action);
                        ui.label(&row.target);
                        ui.label(&row.status);
                        ui.label(&row.details);
                        ui.end_row();
                    }
                });
        });

        None
    }
}

fn csv_export(rows: &[AuditRow]) -> String {
    let mut out = String::from("occurred_at,action,target,status,details\n");
    for row in rows {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_escape(&row.occurred_at),
            csv_escape(&row.action),
            csv_escape(&row.target),
            csv_escape(&row.status),
            csv_escape(&row.details)
        ));
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
