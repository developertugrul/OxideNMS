<div align="center">
  <img src="https://img.icons8.com/color/120/000000/network-cable.png" alt="OxideNMS Logo" width="100"/>
  <h1>OxideNMS</h1>
  <p><strong>A Modern, Fast, and Secure Network Management System Written in Rust</strong></p>

  <p>
    <a href="https://github.com/developertugrul/OxideNMS/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" />
    </a>
    <img src="https://img.shields.io/badge/language-Rust-orange.svg" alt="Made with Rust" />
    <img src="https://img.shields.io/badge/UI-egui-yellow.svg" alt="egui UI" />
  </p>

  <p>
    <a href="README.md">English</a> | <b>Türkçe</b>
  </p>
</div>

## 📌 Hakkında
**OxideNMS**, ağ yöneticileri ve mühendisleri için geliştirilmiş, yüksek performanslı, kurumsal seviyede bir Ağ Yönetim Sistemidir (Network Management System). Gücünü **Rust** programlama dilinden ve **egui** grafik kütüphanesinden alır. Tek bir çalıştırılabilir dosya (executable) olarak gelir ve saniyeler içinde çalışır. Ağınızı güvenle yönetin, yedekleyin, izleyin ve şablonlar oluşturun.

---

## 🚀 Özellikler

### 📊 İzleme ve Yönetim
- **Performans Dashboard:** Cihazların CPU ve RAM kullanımlarını gerçek zamanlı grafiklerle izleyin.
- **SNMP Topoloji Haritası (Cisco CDP/LLDP):** Ağınızdaki cihazları SNMP ile keşfedip bağlantı haritasını çizin.
- **Dahili Syslog Sunucusu (UDP 514):** Ağ cihazlarınızdan gelen logları merkezi olarak toplayıp analiz edin.

### ⚙️ Konfigürasyon ve Otomasyon
- **Toplu Dağıtım (Bulk Deploy):** Yüzlerce cihaza tek tıkla yapılandırma gönderin.
- **Şablon Motoru (Jinja2):** `minijinja` altyapısı sayesinde parametrik ağ konfigürasyon şablonları hazırlayın.
- **Otomatik Yedekleme Servisi:** Ağ cihazlarının konfigürasyonlarını arka planda otomatik olarak çekip arşivleyin.
- **Diff Engine:** Cihaz konfigürasyonlarının eski ve yeni sürümlerini yan yana karşılaştırın (Git diff stili).
- **Otomatik Firmware (IOS) Güncelleme:** TFTP/FTP entegrasyonu ile cihazlarınızın imajlarını kolayca güncelleyin.

### 🛠️ Araçlar ve Utilities
- **Cihaz Yöneticisi:** Tüm ağ donanımlarınızın IP, kimlik bilgileri ve donanım envanterini tutun (SQLite).
- **Subnet Hesaplayıcı (IPv4/IPv6):** Hızlı ve pratik IP alt ağ bölme işlemleri.
- **VLAN Yöneticisi:** Switchler için kolay VLAN havuzu yönetimi.
- **Dahili SSH Terminali:** Harici bir PuTTY/SecureCRT'ye ihtiyaç duymadan cihazlara saniyeler içinde doğrudan bağlanın.

### 🔒 Üst Düzey Güvenlik
- **Master Password Şifreleme (AES-256-GCM):** Veritabanındaki şifreler, belirlediğiniz ana parola ile şifrelenir. Cihazlarınıza ait parolalar açık metin (plaintext) olarak asla diskte tutulmaz!

---

## 📥 Kurulum

OxideNMS tek bir çalıştırılabilir (`.exe`, `.AppImage`, vb.) dosyadır. Bağımlılık kurmanıza (Python, Java vb.) gerek yoktur.

### Kaynaktan Derleme
Sisteminizde [Rust](https://rustup.rs/) kuruluysa:

```bash
# Repoyu klonlayın
git clone https://github.com/developertugrul/OxideNMS.git
cd OxideNMS

# Projeyi derleyin ve çalıştırın
cargo run --release
```
Derleme tamamlandıktan sonra oluşturulan `target/release/oxidenms.exe` dosyasını istediğiniz yere taşıyarak doğrudan çalıştırabilirsiniz.

---

## 📚 Kullanım
1. **İlk Açılış:** Uygulamayı açtığınızda sizden bir **Master Password** istenir. Bu parola, cihazlarınızın şifrelerini veritabanında güvenle saklamak için kullanılır.
2. **Cihaz Ekleme:** Sol menüden *Cihaz Yöneticisi*'ne girip cihazlarınızı ekleyin.
3. **Yönetim:** İstediğiniz aracı seçin ve ağınızı yönetmeye başlayın!

---

## 👨‍💻 Geliştirme ve Katkı
Bu proje tamamen **Açık Kaynak** olup katkılarınıza açıktır. PR (Pull Request) ve Issue bildirimlerinizi bekliyoruz!

1. Projeyi fork'layın.
2. Kendi feature branch'inizi oluşturun (`git checkout -b feature/MuthisOzellik`).
3. Değişikliklerinizi commit'leyin (`git commit -m 'feat: MuthisOzellik eklendi'`).
4. Branch'inize push yapın (`git push origin feature/MuthisOzellik`).
5. Pull Request açın.

---

## 📜 Lisans
Bu proje **MIT Lisansı** ile lisanslanmıştır. Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakabilirsiniz.
