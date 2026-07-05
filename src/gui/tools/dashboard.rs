use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::Language;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

pub struct DashboardTool {
    time: f64,
}

impl Default for DashboardTool {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}

impl DashboardTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ToolScreen for DashboardTool {
    fn id(&self) -> &'static str {
        "dashboard"
    }

    fn icon(&self) -> &'static str {
        "📈"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Performans Dashboard"
    }

    fn draw(&mut self, ui: &mut egui::Ui, _dil: Language) -> Option<ToolEvent> {
        self.time += ui.input(|i| i.stable_dt) as f64;

        ui.heading("Ağ Cihazları Gerçek Zamanlı Performans Grafikleri");
        ui.add_space(10.0);

        let n = 120;

        // Simüle edilmiş CPU kullanımı
        let cpu_points: PlotPoints = (0..n)
            .map(|i| {
                let x = i as f64;
                let y = 50.0
                    + 20.0 * (self.time + x / 10.0).sin()
                    + 10.0 * (self.time * 3.0 + x / 5.0).cos();
                [x, y]
            })
            .collect();
        let cpu_line = Line::new(cpu_points)
            .name("Router CPU (%)")
            .width(2.0)
            .color(egui::Color32::from_rgb(100, 200, 100));

        // Simüle edilmiş RAM kullanımı
        let ram_points: PlotPoints = (0..n)
            .map(|i| {
                let x = i as f64;
                let y = 60.0 + 15.0 * (self.time * 0.5 + x / 20.0).cos();
                [x, y]
            })
            .collect();
        let ram_line = Line::new(ram_points)
            .name("Switch RAM (%)")
            .width(2.0)
            .color(egui::Color32::from_rgb(200, 100, 100));

        Plot::new("performance_plot")
            .view_aspect(2.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(cpu_line);
                plot_ui.line(ram_line);
            });

        ui.add_space(15.0);
        ui.label("Not: Grafikler SNMP get ve trap verileriyle beslenebilir. Mevcut görünüm siber güvenlik ve ağ izleme için bir demo prototipidir.");

        ui.ctx().request_repaint(); // Grafiğin sürekli akması için

        None
    }
}
