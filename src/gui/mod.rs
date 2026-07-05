//! Masaüstü grafik arayüz (GUI) katmanı — `egui`/`eframe` ile.
//!
//! Bu dosya GUI'nin "kabuğudur": pencereyi açar, sol/orta panelleri ve
//! sürüm (güncelleme) ekranlarını yönetir. Araçların KENDİSİ `tools/`
//! altındadır ve ortak `ToolScreen` trait'ini uygular. Tüm metinler `i18n::t`
//! üzerinden seçili dilde gelir.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui;

pub mod tools;

use tools::{
    AuditLogTool, BackupTool, BulkDeployTool, ConfigHistoryTool, DashboardTool, DeviceManagerTool,
    DiffTool, DiscoveryTool, FirmwareTool, SecurityTool, SettingsTool, SnmpMapTool, SshTool,
    SubnetTool, SyslogTool, TemplateTool, ToolEvent, ToolScreen, TopologyTool, VlanTool,
};

use crate::i18n::{Language, Message, t};
use crate::settings::{AppSettings, Theme};
use crate::update::{self, UpdateStatus, VersionManifest};

/// GUI'yi başlatır ve pencereyi açar.
pub fn run() -> eframe::Result {
    // Ayarları diskten yükle (yoksa varsayılan oluşturulur).
    let settings = AppSettings::load();
    if let Some(yol) = AppSettings::dosya_yolu() {
        println!("Ayar dosyası: {}", yol.display());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Tam ekran genişliği: pencere maksimize açılır.
            .with_maximized(true)
            // Sabit: kullanıcı boyutlandıramaz.
            .with_resizable(true)
            .with_min_inner_size([1000.0, 650.0])
            // Maksimize desteklenmezse kullanılacak yedek boyut.
            .with_inner_size([1100.0, 700.0])
            .with_title(t(settings.dil, Message::AppName)),
        ..Default::default()
    };

    eframe::run_native(
        "OxideNMS",
        options,
        Box::new(move |cc| Ok(Box::new(CiscoApp::new(&cc.egui_ctx, settings)))),
    )
}

/// 17 dilin tüm yazı sistemlerini göstermek için gömülü fontları kurar:
///   - DejaVu Sans        : Latin + Kiril + Yunan (birincil)
///   - Noto Sans Devanagari: Hintçe
///   - Noto Sans SC       : Çince (CJK)
///
/// egui, bir glifi ilk fontta bulamazsa fallback zincirinde sonrakine bakar;
/// bu yüzden script'e özel fontları yedek olarak eklemek yeterli.
fn fontlari_kur(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "dejavu".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/DejaVuSans.ttf"
        ))),
    );
    fonts.font_data.insert(
        "noto_deva".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSansDevanagari.ttf"
        ))),
    );
    fonts.font_data.insert(
        "noto_sc".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSansSC.ttf"
        ))),
    );

    // Proportional: DejaVu birincil; Devanagari ve CJK yedek.
    let prop = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    prop.insert(0, "dejavu".to_owned());
    prop.push("noto_deva".to_owned());
    prop.push("noto_sc".to_owned());

    // Monospace: hizayı bozmamak için varsayılan mono birincil; hepsi yedek.
    let mono = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();
    mono.push("dejavu".to_owned());
    mono.push("noto_deva".to_owned());
    mono.push("noto_sc".to_owned());

    ctx.set_fonts(fonts);
}

/// Sürüm kontrolünün arka plandaki aşaması.
enum UpdateState {
    Checking,
    Ready(UpdateStatus),
    /// Politikamız "fail-open": uygulama normal çalışır.
    Failed(String),
}

/// Uygulamanın tüm arayüz durumu (state).
struct CiscoApp {
    tools: Vec<Box<dyn ToolScreen>>,
    secili: usize,
    /// Seçili arayüz dili (canlı; Ayarlar'dan değiştirilebilir).
    dil: Language,
    update_state: UpdateState,
    update_rx: Option<Receiver<Result<UpdateStatus, String>>>,
}

impl CiscoApp {
    /// Uygulamayı kurar, fontu/temayı/dili uygular ve sürüm kontrolünü başlatır.
    fn new(ctx: &egui::Context, settings: AppSettings) -> Self {
        // Uygulama yüklenirken veritabanının ve tabloların kurulduğundan emin ol
        if let Err(e) = crate::db::get_connection() {
            eprintln!("Veritabanı başlatılamadı: {}", e);
        }

        fontlari_kur(ctx);

        let mut visuals = match settings.tema {
            Theme::Koyu => egui::Visuals::dark(),
            Theme::Acik => egui::Visuals::light(),
        };

        // UI Enhancements

        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(8);

        if matches!(settings.tema, Theme::Koyu) {
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 27, 30);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 38, 42);
        }

        ctx.set_visuals(visuals);

        // Styling configuration for padding and spacing
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        ctx.set_style(style);

        // Dili ayardan al (settings taşınmadan önce; Language Copy'dir).
        let dil = settings.dil;

        let (tx, rx) = channel();
        let manifest_url = settings.effective_manifest_url();

        let bg_ctx = ctx.clone();
        std::thread::spawn(move || {
            let result =
                update::check(manifest_url, env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string());
            let _ = tx.send(result);
            bg_ctx.request_repaint();
        });

        Self {
            tools: vec![
                Box::new(DashboardTool),
                Box::new(AuditLogTool::default()),
                Box::new(DeviceManagerTool::default()),
                Box::new(DiscoveryTool::default()),
                Box::new(SecurityTool::default()),
                Box::new(BackupTool::default()),
                Box::new(ConfigHistoryTool::default()),
                Box::new(BulkDeployTool::default()),
                Box::new(DiffTool::default()),
                Box::new(SshTool::default()),
                Box::new(SyslogTool::default()),
                Box::new(SnmpMapTool::default()),
                Box::new(TopologyTool::default()),
                Box::new(SubnetTool::default()),
                Box::new(VlanTool::default()),
                Box::new(TemplateTool::default()),
                Box::new(FirmwareTool::default()),
                Box::new(SettingsTool::new(settings)),
            ],
            secili: 0,
            dil,
            update_state: UpdateState::Checking,
            update_rx: Some(rx),
        }
    }

    /// Arka plandan sonuç geldiyse durumu günceller.
    fn surum_sonucunu_al(&mut self) {
        if let Some(rx) = &self.update_rx
            && let Ok(result) = rx.try_recv()
        {
            self.update_state = match result {
                Ok(status) => UpdateState::Ready(status),
                Err(mesaj) => UpdateState::Failed(mesaj),
            };
            self.update_rx = None;
        }
    }
}

impl eframe::App for CiscoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.surum_sonucunu_al();

        // 1) Kontrol sürüyorsa: bekleme ekranı göster.
        if matches!(self.update_state, UpdateState::Checking) {
            self.bekleme_ekrani(ctx);
            return;
        }

        // 2) Zorunlu güncelleme varsa: kilit ekranı göster ve DUR.
        if let UpdateState::Ready(UpdateStatus::Mandatory(m)) = &self.update_state {
            let manifest = m.clone();
            self.kilit_ekrani(ctx, &manifest);
            return;
        }

        // 3) İsteğe bağlı güncelleme varsa: üstte uyarı.
        if let UpdateState::Ready(UpdateStatus::Optional(m)) = &self.update_state {
            let manifest = m.clone();
            self.istege_bagli_uyari(ctx, &manifest);
        }

        // Denetim başarısızsa (fail-open): altta küçük uyarı.
        if let UpdateState::Failed(mesaj) = &self.update_state {
            let mesaj = mesaj.clone();
            self.durum_cubugu(ctx, &mesaj);
        }

        self.ust_bar(ctx);
        self.sol_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let secili = self.secili;
            let dil = self.dil;
            if let Some(olay) = self.tools[secili].draw(ui, dil) {
                match olay {
                    // Language değişti: uygulama geneline anında uygula.
                    ToolEvent::LanguageSelected(yeni) => self.dil = yeni,
                    // Diff ekranına yönlendirme
                    ToolEvent::SwitchToDiff {
                        old_config,
                        new_config,
                    } => {
                        let mut diff_index = None;
                        for (i, tool) in self.tools.iter_mut().enumerate() {
                            if tool.id() == "diff" {
                                tool.receive_data(old_config.clone(), new_config.clone());
                                diff_index = Some(i);
                            }
                        }
                        if let Some(i) = diff_index {
                            self.secili = i;
                        }
                    }
                }
            }
        });
    }
}

// --- Ana yerleşim ---

impl CiscoApp {
    fn ust_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("ust_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(t(self.dil, Message::AppName));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                });
            });
        });
    }

    fn sol_panel(&mut self, ctx: &egui::Context) {
        let dil = self.dil;
        // Araç adlarını önceden topla (borrow sorununu önlemek için).
        let adlar: Vec<(&'static str, &'static str)> =
            self.tools.iter().map(|a| (a.icon(), a.name(dil))).collect();

        egui::SidePanel::left("tools")
            .resizable(false)
            .exact_width(220.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);

                // Kategorileri tanımla: (Başlık, [Tool İndeksleri])
                let kategoriler = vec![
                    ("Operasyon", vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                    ("Izleme & Topoloji", vec![10, 11, 12]),
                    ("Planlama & Lab", vec![13, 14, 15, 16]),
                    ("Sistem", vec![17]),
                ];

                for (kat_adi, indeksler) in kategoriler {
                    ui.label(egui::RichText::new(kat_adi).strong().weak().size(14.0));
                    ui.add_space(4.0);

                    for i in indeksler {
                        if let Some((icon, name)) = adlar.get(i) {
                            let label = format!("{}  {}", icon, name);
                            // Seçili item için biraz daha yüksek padding/font verebiliriz
                            let selected = self.secili == i;
                            let text = if selected {
                                egui::RichText::new(label).strong()
                            } else {
                                egui::RichText::new(label)
                            };

                            if ui.add(egui::SelectableLabel::new(selected, text)).clicked() {
                                self.secili = i;
                            }
                        }
                    }
                    ui.add_space(12.0);
                }
            });
    }
}

// --- Sürüm / güncelleme ekranları ---

impl CiscoApp {
    fn bekleme_ekrani(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(160.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label(t(self.dil, Message::CheckingUpdates));
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }

    fn kilit_ekrani(&self, ctx: &egui::Context, manifest: &VersionManifest) {
        let dil = self.dil;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.heading(egui::RichText::new(t(dil, Message::UpdateRequired)).size(26.0));
                ui.add_space(12.0);
                ui.label(t(dil, Message::VersionUnsupported));
                ui.add_space(16.0);

                egui::Grid::new("kilit_surum")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(t(dil, Message::YourVersion)).weak());
                        ui.label(
                            egui::RichText::new(env!("CARGO_PKG_VERSION"))
                                .monospace()
                                .color(egui::Color32::from_rgb(220, 80, 80)),
                        );
                        ui.end_row();
                        ui.label(egui::RichText::new(t(dil, Message::RequiredVersion)).weak());
                        ui.label(
                            egui::RichText::new(&manifest.latest_version)
                                .monospace()
                                .strong(),
                        );
                        ui.end_row();
                    });

                if let Some(notes) = &manifest.notes {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(notes).italics().weak());
                }

                ui.add_space(24.0);
                let buton = egui::Button::new(
                    egui::RichText::new(t(dil, Message::DownloadUpdate)).size(16.0),
                )
                .min_size(egui::vec2(220.0, 40.0));
                if ui.add(buton).clicked() {
                    ctx.open_url(egui::OpenUrl::new_tab(&manifest.download_url));
                }
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(t(dil, Message::DownloadInBrowser))
                        .small()
                        .weak(),
                );
            });
        });
    }

    fn durum_cubugu(&self, ctx: &egui::Context, mesaj: &str) {
        egui::TopBottomPanel::bottom("durum").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t(self.dil, Message::UpdateCheckFailed))
                        .small()
                        .color(egui::Color32::from_rgb(200, 150, 60)),
                );
                ui.label(egui::RichText::new(mesaj).small().weak());
            });
        });
    }

    fn istege_bagli_uyari(&self, ctx: &egui::Context, manifest: &VersionManifest) {
        let dil = self.dil;
        egui::TopBottomPanel::top("istege_bagli").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mesaj = t(dil, Message::NewVersionAvailable)
                    .replace("{0}", &manifest.latest_version)
                    .replace("{1}", env!("CARGO_PKG_VERSION"));
                ui.label(mesaj);
                if ui.button(t(dil, Message::Download)).clicked() {
                    ctx.open_url(egui::OpenUrl::new_tab(&manifest.download_url));
                }
            });
        });
    }
}
