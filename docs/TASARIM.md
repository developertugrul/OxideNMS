# Cisco Ağ Araçları — Sistem Tasarım Dökümanı

> Bu döküman projenin **anayasasıdır**. Her yeni faz ve iyileştirme buradaki
> ilkelere ve mimariye uymalıdır. Kod değiştikçe bu döküman da güncellenir.

Sürüm: taslak 1 · Son güncelleme: 2026-07

---

## 1. Vizyon ve Kapsam

**Tek cümle:** CCNA/Cisco bilgisini, ağı yöneten/izleyen/denetleyen güvenilir
araçlara dönüştüren **%100 Rust** masaüstü uygulaması.

- **Dil/Yığın:** Sadece Rust. Laravel/PHP/harici backend **yok**.
- **Dağıtım:** Tek çalıştırılabilir masaüstü uygulaması (`.exe`), GUI (egui).
- **Nihai ürün:** Kurumsal "Cisco Network Auditor" — cihazlara bağlanıp
  konfigürasyon yedekleyen, değişiklikleri bulan, hatalı VLAN/ACL/trunk/routing
  ayarlarını raporlayan, topoloji çıkaran bir ağ denetim aracı.
- **Yöntem:** Çok küçük, çalışır fazlar. Her faz tek başına değer üretir.

**Kapsam dışı (şimdilik):** bulut/web paneli, mobil, çok kullanıcılı sunucu.
İleride istenirse aynı `network/` domaini üzerine ayrı bir arayüz olarak eklenir.

---

## 2. Yol Gösterici İlkeler

1. **Katmanlı mimari / separation of concerns.** Domain (saf mantık) hiçbir
   arayüzü bilmez. Arayüzler (CLI/GUI) domaini çağırır. Bir arayüz eklemek
   veya değiştirmek domaini bozmaz.
2. **OOP-uyumlu, Rust-idiomatik.** Kalıtım yok; `struct` (veri) + `impl`
   (davranış) + `trait` (arayüz/sözleşme) + `mod` (ad alanı) kullanılır.
3. **Geçersiz durum temsil edilemez.** Doğrulama constructor'da yapılır
   (`Subnet::parse`), geçersiz veri asla nesneye dönüşmez.
4. **Hatalar tip-güvenli.** `panic!` yerine `Result` + merkezi hata enum'ları.
5. **Saf mantık test edilir.** I/O'dan ayrılmış her karar fonksiyonu
   `#[cfg(test)]` ile test edilir (`update::decide` gibi).
6. **Küçük, açıklanabilir adımlar.** Her değişiklik anlaşılır ve geri alınabilir.
7. **Türkçe.** Kod yorumları, döküman ve arayüz metinleri Türkçe.

---

## 3. Mimari Genel Bakış

Üç mantıksal katman:

```
┌─────────────────────────────────────────────────────────┐
│  ARAYÜZ KATMANI (interface)                              │
│  cli/            gui/                                     │
│  - kullanıcıdan girdi alır                               │
│  - domaini çağırır                                       │
│  - sonucu gösterir   (mantık YOK)                        │
└───────────────┬─────────────────────────────────────────┘
                │ çağırır
┌───────────────▼─────────────────────────────────────────┐
│  SERVİS KATMANI (uygulama servisleri)                    │
│  update/   (ileride: device/, storage/, ssh/)           │
│  - I/O yapar (HTTP, dosya, ağ), domaini orkestre eder    │
└───────────────┬─────────────────────────────────────────┘
                │ kullanır
┌───────────────▼─────────────────────────────────────────┐
│  DOMAIN KATMANI (saf mantık, I/O yok)                    │
│  network/  (subnet; ileride: config, diff, topoloji)    │
│  error.rs  (ortak hata tipleri)                          │
└─────────────────────────────────────────────────────────┘
```

**Bağımlılık yönü tek yönlüdür:** yukarıdan aşağıya. Domain, servis veya
arayüz hakkında hiçbir şey bilmez. Bu kuralı bozmayın.

> Not: Bugün `update/` hem I/O hem karar içeriyor ama karar kısmı (`decide`)
> saf ve test edilebilir biçimde ayrık. Servis katmanı büyüdükçe (SSH, storage)
> bu ayrım netleşecek.

---

## 4. Modül Detayları

### `network/` — Ağ Domaini (saf mantık)
- `subnet.rs`: `Subnet` nesnesi. Sorumluluğu: bir IPv4 + prefix'ten
  network/broadcast/host aralığı/host sayısı hesaplamak.
  - Constructor: `Subnet::parse("192.168.1.10/24")`, `Subnet::new(ip, prefix)`.
  - Getter: `ip()`, `prefix()`. Hesaplama: `mask()`, `network()`,
    `broadcast()`, `first_host()`, `last_host()`, `usable_hosts()`.
  - `/31` ve `/32` özel durumları ele alınmış.
- `guvenlik.rs`: Cisco config güvenlik denetimi (hardening). `denetle(config)
  -> Vec<Bulgu>`; `Seviye` (Kritik/Uyarı/Bilgi), `BulguKodu` (7 kural: telnet,
  enable secret yok, SNMP public/private, password-encryption yok, HTTP server,
  zayıf parola). Saf mantık + birim testli. GUI: `araclar/guvenlik.rs`.
- **Gelecek:** `config.rs` (Cisco config modeli), `diff.rs` (config farkı),
  `topology.rs` (CDP/LLDP komşuluk grafiği), daha fazla güvenlik kuralı.

### `update/` — Sürüm Kontrolü Servisi
- `check(url, current)`: manifest'i indirir (ureq), JSON çözer (serde),
  kararı `decide`'a devreder.
- `decide(current, manifest)`: **saf** karar mantığı → `UpdateStatus`.
- `VersionManifest`: `latest_version`, `minimum_version`, `download_url`, `notes`.
- `UpdateStatus`: `UpToDate` | `Optional(manifest)` | `Mandatory(manifest)`.
- Detay için §6.

### `cli/` — Komut Satırı Arayüzü
- `clap` türetmeli (derive) yapı. `Cli` + `Commands` enum.
- Şu an: `subnet <cidr>`. Yeni özellik = yeni `Commands` varyantı.
- Sadece köprü: domaini çağırır, `println!` ile basar.

### `gui/` — Masaüstü Arayüz (eframe/egui)
- `CiscoApp`: tüm arayüz durumu (state). `eframe::App::update` her karede çizer.
- Sol panel: araç listesi (`Tool` enum). Orta panel: seçili aracın ekranı.
- Sürüm ekranları: bekleme (spinner), kilit (zorunlu), üst uyarı (isteğe bağlı),
  alt durum çubuğu (denetim başarısız).

### `i18n/` — Çoklu Dil (Uluslararasılaştırma)
- `Dil` enum (17 dil), `Mesaj` enum (tip-güvenli metin anahtarları),
  `t(dil, mesaj)` fonksiyonu. Çevrilmemiş diller İngilizce'ye düşer.
- Arayüz asla sabit metin yazmaz; her zaman `t(dil, Mesaj::X)` çağırır.
- Parametreli metinler `{0}`/`{1}` yer tutucu içerir, çağıran `.replace` eder.
- Dil `settings.toml`'da saklanır; Ayarlar'daki seçiciyle canlı değişir
  (`AracOlayi::DilSecildi` olayıyla kabuğa bildirilir).
- `mod.rs` = mekanizma (Dil, Mesaj, t dağıtım), `diller.rs` = veri (her dil
  bir fonksiyon, `Mesaj` üzerinde exhaustive match → eksik çeviri derleme hatası).
- **Durum: 17 dilin hepsi tam.** Fontlar: DejaVu (Latin/Kiril/Yunan) +
  Noto Devanagari (Hintçe) + Noto SC (Çince), fallback zinciriyle gömülü.
  Azınlık dilleri için anadil kontrolü önerilir.

### `settings.rs` — Uygulama Ayarları
- `AppSettings` (`manifest_url`, `tema`, `dil`) → `settings.toml` (OS config
  klasörü, Windows'ta `%APPDATA%\cisco\`).
- `load()` panik atmaz: dosya yoksa oluşturur, bozuksa varsayılana düşer.
- `effective_manifest_url()`: `CISCO_MANIFEST_URL` ortam değişkeni > dosya > varsayılan.

> Font: GUI, Türkçe karakterleri garanti göstermek için `assets/fonts/DejaVuSans.ttf`
> fontunu gömer (`gui::fontlari_kur`, `include_bytes!`). Proportional ailesinde
> birincil, Monospace'te yedek. Böylece varsayılan fonta bağımlı kalınmaz.

### `gui/araclar/` — GUI Araçları (trait tabanlı)
- `AracEkrani` trait'i (`ad()`, `ciz()`) = her aracın sözleşmesi.
- `SubnetAraci` bu trait'i uygular. `CiscoApp` araçları
  `Vec<Box<dyn AracEkrani>>` olarak tutar (dynamic dispatch).
- **Yeni araç eklemek:** yeni struct + `AracEkrani` uygula + `CiscoApp::new`
  listesine ekle. Başka yer değişmez.

### `error.rs` — Merkezi Hatalar
- `NetworkError` (subnet/ip), `UpdateError` (http/io/parse/version).
- `thiserror` ile her hataya Türkçe, kullanıcıya gösterilebilir mesaj.

### `lib.rs` / `main.rs`
- `lib.rs`: kütüphane kökü, tüm modüller `pub`. Testler ve arayüzler bunu kullanır.
- `main.rs`: ince giriş. Şu an `cisco::gui::run()`. İleride argümana göre
  CLI/GUI seçimi buraya eklenebilir.

---

## 5. Veri Akışı Örnekleri

**Subnet hesaplama (GUI):**
```
Kullanıcı "192.168.1.10/24" yazar
  → gui::subnet_ekrani her karede Subnet::parse çağırır
    → başarılı: sonuç tablosu çizilir
    → hata: kırmızı hata satırı (NetworkError mesajı)
```

**Açılışta sürüm kontrolü:**
```
CiscoApp::new → arka planda thread: update::check(manifest_url, sürüm)
  → sonuç kanal (channel) ile UI'ye döner + request_repaint
    → Mandatory  : kilit ekranı (uygulama kullanılamaz)
    → Optional   : üst uyarı + normal uygulama
    → UpToDate   : normal uygulama
    → hata       : normal uygulama + alt durum çubuğu (fail-open)
```

---

## 6. Zorunlu Güncelleme Sistemi

**Amaç:** yeni sürüm çıkınca eski sürümdeki kullanıcıları güncellemeye zorlamak.

**Model (seçilen):** *Kilitle + indirme sayfası.* (Otomatik indirme / self-update
değil — bu ileride bir iyileştirme.)

**Manifest (senin kontrolündeki bir URL'de duran JSON):**
```json
{
  "latest_version": "1.3.0",
  "minimum_version": "1.3.0",
  "download_url": "https://.../cisco-setup.exe",
  "notes": "Kritik güvenlik güncellemesi"
}
```

**Karar tablosu:**

| Koşul | Sonuç |
|-------|-------|
| sürüm ≥ `minimum_version` ve = `latest_version` | UpToDate (normal) |
| sürüm ≥ `minimum_version` ama < `latest_version` | Optional (uyarı + normal) |
| sürüm < `minimum_version` | **Mandatory (kilit)** |
| denetim başarısız (offline) | fail-open (normal + not) |

**Yönetim akışı:**
1. Yeni `.exe` yayınla.
2. Cargo.toml `version`'ı artır.
3. Hosted `latest.json`'da `latest_version`'ı güncelle.
4. Zorunlu yapmak istiyorsan `minimum_version`'ı da yükselt → eski sürümler kilitlenir.

**Yapılandırma:** URL `update::DEFAULT_MANIFEST_URL` (placeholder) veya
`CISCO_MANIFEST_URL` ortam değişkeni. Yerel test: `assets/latest.example.json`
+ `python -m http.server`.

**Politika notu:** `Failed` (offline) durumunda fail-open. Daha katı istenirse
`gui::UpdateState::Failed` dalı kilitlemeye çevrilebilir — ama offline
kullanıcıyı cezalandırır, dikkatli olunmalı.

---

## 7. Teknoloji Yığını ve Gerekçeler

| Bağımlılık | Ne için | Neden bu |
|------------|---------|----------|
| `eframe`/`egui` | Masaüstü GUI | Immediate-mode, hızlı öğrenilir, araç/dashboard'a uygun, tek exe |
| `clap` | CLI ayrıştırma | Rust'ta fiili standart; `--help`/`--version` bedava |
| `thiserror` | Hata tipleri | Kütüphane hataları için tip-güvenli, mesajlı enum'lar |
| `ureq` | HTTP (manifest) | Bloklayan, hafif; arka plan thread'inde basit kullanım |
| `serde`/`serde_json` | JSON çözme | Manifest ayrıştırma; ekosistem standardı |
| `semver` | Sürüm karşılaştırma | Doğru semver mantığı (elle karşılaştırmadan kaçınmak) |
| `toml` | Ayar dosyası | İnsan-okunur, düzenlenebilir ayar formatı |
| `dirs` | OS config klasörü | Platform-doğru ayar konumu (%APPDATA% vb.) |

**Gelecekte muhtemel:** `ssh2` veya `russh` (SSH), `tokio` (async toplama),
`rusqlite`/`sqlx` (yerel saklama), `regex` (config parse), `ratatui` (opsiyonel TUI).

---

## 8. Konvansiyonlar

- **İsimlendirme:** tip `PascalCase`, fonksiyon/değişken `snake_case`,
  sabit `SCREAMING_SNAKE_CASE`. Domain isimleri Türkçe olabilir (`satir`, `yazdir`).
- **Hata yönetimi:** kütüphane katmanında `Result` + `thiserror` enum'ları.
  Arayüz katmanı hatayı kullanıcıya gösterir; `unwrap`/`panic` sadece
  gerçekten imkânsız durumlarda.
- **Test:** saf karar fonksiyonları için `#[cfg(test)]` birim testi. I/O'yu
  test etmek için mantığı I/O'dan ayır (örn. `decide` / `check`).
- **Yorumlar:** neden'i açıkla, ne'yi değil. Türkçe.
- **Modül eklerken:** domain mi servis mi arayüz mi karar ver, doğru katmana koy,
  `lib.rs`'e `pub mod` ekle.

---

## 9. Genişleme Noktaları (gelecek fazlar için tasarım)

Bunlar henüz KODLANMADI; ileride buraya göre inşa edilecek.

- **`Tool` soyutlaması:** GUI'de her araç ortak bir `trait` (örn. `AracEkrani`
  { `ad()`, `ciz(ui)` }) uygularsa, yeni araç eklemek = yeni tip + listeye kayıt.
  Şu an `Tool` bir enum; araç sayısı artınca trait'e geçmek iyileştirme adayı.
- **Cihaz envanteri (`device/`):** IP, kullanıcı adı, kimlik. Yerel saklama
  (SQLite veya şifreli dosya). Kimlik bilgileri asla düz metin saklanmaz.
- **SSH toplayıcı (`ssh/` servis):** cihaza bağlanır, `show running-config`
  vb. çalıştırır, çıktıyı domaine (`config.rs`) verir.
- **Saklama (`storage/` servis):** config sürümleri, diff geçmişi, alarmlar.
- **Config domaini (`network/config.rs`, `diff.rs`):** Cisco config'i yapısal
  modele çevirir, iki sürümü karşılaştırır, compliance kurallarını uygular.
- **Ayarlar:** `config.toml` benzeri kullanıcı ayarları (manifest URL, tema,
  cihaz listesi yolu) — sabit yerine yapılandırılabilir.

---

## 10. Yol Haritası

| Faz | İçerik | Cihaz gerekli? | Durum |
|-----|--------|----------------|-------|
| 0 | Kurulum, iskelet | Hayır | ✅ Bitti |
| 1 | Subnet hesaplayıcı (CLI + GUI) | Hayır | ✅ Bitti |
| — | Zorunlu güncelleme sistemi | Hayır | ✅ Bitti |
| 2 | VLAN/Subnet planlayıcı (+ Cisco config çıktısı) | Hayır | ⏭️ Sıradaki |
| 3 | Cisco config dosyası parse (VLAN/interface çıkar) | Hayır | ⬜ |
| 4 | İki config diff (ne değişmiş?) | Hayır | ⬜ |
| 5 | Sanal lab (Packet Tracer/GNS3) + SSH ile config çekme | Evet (simülatör) | ⬜ |
| 6 | Config saklama + otomatik diff | Evet | ⬜ |
| 7 | Compliance kuralları (yanlış VLAN/ACL/trunk yakala) | Evet | ⬜ |
| 8+ | Topoloji, health monitor, alarmlar, raporlama | Evet | ⬜ |

**Not:** Faz 5'ten önce gerçek Cisco donanımı GEREKMEZ; ücretsiz simülatör
(Cisco Packet Tracer / GNS3 / EVE-NG) kullanılacak.

---

## 11. İyileştirme Adayları (biriktir, sırası gelince yap)

- ~~GUI'de `Tool` enum'unu trait tabanlı çoklu-araç mimarisine çevirmek.~~ ✅ Yapıldı
- ~~Manifest URL'ini ve tema gibi ayarları bir `settings.toml`'a taşımak.~~ ✅ Yapıldı
- `network/subnet.rs` için birim testleri (şu an testler `update` ve `settings`'te).
- `main.rs`'e argüman varsa CLI, yoksa GUI seçimi eklemek.
- ~~GUI'ye Türkçe karakter için özel font gömmek.~~ ✅ Yapıldı (DejaVu Sans,
  `assets/fonts/`, `gui::fontlari_kur`). Tüm GUI metinleri düzgün Türkçe.
- ~~GUI içinden ayarları (tema, manifest URL) düzenleyebilen bir "Ayarlar" ekranı.~~ ✅ Yapıldı
- Zorunlu güncellemede indirme dosyasının imza/hash doğrulaması (güvenlik).
- İleride: self-update (self_update crate) ile otomatik indirme.
```

