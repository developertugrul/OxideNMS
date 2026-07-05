# OxideNMS 🚀

![Rust](https://img.shields.io/badge/rust-v1.80%2B-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=flat-square)

**OxideNMS**, ağ mühendisleri ve sistem yöneticileri için tamamen [Rust](https://www.rust-lang.org/) ile geliştirilmiş, ultra hızlı, güvenli ve açık kaynaklı bir **Ağ Yönetim Sistemi (NMS)**'dir. 

Geleneksel ağ yönetim araçlarının karmaşıklığından kurtulun. OxideNMS, tek bir güvenli çatı altında tüm ağ cihazlarınızı yönetmenizi, yapılandırmanızı ve izlemenizi sağlar. EGUI tabanlı modern grafik arayüzü ile çapraz platform çalışır.

---

## 🔥 Temel Özellikler (Features)

- **🔒 Güvenli Cihaz Kasası (Device Vault)**
  - Cihaz şifreleriniz ve SSH anahtarlarınız SQLite veritabanında düz metin olarak **TUTULMAZ**.
  - Tüm kimlik bilgileri, uygulamanın başında sizin belirlediğiniz bir **Master Password** ile PBKDF2 ve **AES-256-GCM** kullanılarak askeri standartlarda şifrelenir.
- **🗺️ Dinamik SNMP Topoloji Haritası**
  - Ağınızdaki cihazların durumunu (UP/DOWN) anlık olarak izleyin.
  - Sürüklenebilir interaktif arayüz ile kendi topolojinizi çizin.
- **⚡ Toplu Dağıtım (Bulk Deployment)**
  - Aynı anda 50 farklı cihaza komut mu göndermeniz gerekiyor? 
  - OxideNMS, Rust'ın muazzam eşzamanlılık (multi-threading) gücüyle yüzlerce cihaza saniyeler içinde paralel SSH bağlantısı kurarak komutlarınızı işletir.
- **⏱️ Otomatik Yedekleme Servisi (Scheduled Backups)**
  - Arka planda çalışan zamanlayıcı servis sayesinde ağ cihazlarınızın `running-config` yedeği, sizin belirlediğiniz saat periyotlarında otomatik olarak alınır ve veritabanına kaydedilir.
- **🔍 Config Diff & Güvenlik Analizi**
  - İki farklı konfigürasyon dosyasını karşılaştırın (Diff).
  - Cihazınızdaki güvenlik açıklarını saniyeler içinde tespit eden otomatik Denetim Modülü (Security Audit).
- **🧮 Ağ Planlama Araçları**
  - Hızlı IPv4 Subnet hesaplayıcı (Network, Broadcast, Host aralıkları).
  - Dinamik VLAN şablon ve komut üretici.

---

## 🛠️ Kurulum (Installation)

OxideNMS, ek bir bağımlılık olmadan çalışabilen tek bir çalıştırılabilir dosya (binary) olarak gelir. 

### Hazır Sürümler (Pre-built Binaries)
GitHub üzerindeki [Releases](https://github.com/developertugrul/OxideNMS/releases) sayfasından işletim sisteminize uygun (Windows `.exe`, macOS veya Linux) sürümü indirebilirsiniz. Sürümler GitHub Actions tarafından otomatik derlenmektedir.

### Kaynaktan Derleme (Build from Source)
Proje açık kaynak olduğu için kendi ortamınızda anında derleyebilirsiniz:

1. [Rust](https://rustup.rs/) kurulu olduğundan emin olun.
2. Repoyu bilgisayarınıza klonlayın:
   ```bash
   git clone https://github.com/yourusername/oxidenms.git
   cd oxidenms
   ```
3. Projeyi derleyin ve çalıştırın:
   ```bash
   cargo run --release
   ```

---

## 📚 Kullanım (Usage)

1. **İlk Çalıştırma:** Uygulamayı açtığınızda `Cihaz Yönetimi` sekmesine gidin.
2. **Kasa (Vault) Kurulumu:** Cihazlarınızı güvenle saklamak için bir **Master Password** belirleyin.
3. **Cihaz Ekleme:** Ağ cihazlarınızın IP, Kullanıcı Adı ve Şifre/Anahtar bilgilerini sisteme kaydedin.
4. **Toplu Yönetim:** `Bulk Deploy` veya `Topoloji Haritası` sekmelerine geçerek otomasyonun keyfini çıkarın!

---

## 🏗️ Katkıda Bulunma (Contributing)

Bu proje açık kaynak topluluğu için geliştirilmiştir. Katkılarınızı büyük bir memnuniyetle bekliyoruz!
1. Bu depoyu çatallayın (Fork).
2. Yeni özelliğiniz için bir dal (branch) oluşturun (`git checkout -b ozellik/YeniGelistirme`).
3. Değişikliklerinizi kaydedin (`git commit -m 'Yeni özellik eklendi'`).
4. Dalınızı (branch) gönderin (`git push origin ozellik/YeniGelistirme`).
5. Bir Çekme İsteği (Pull Request) oluşturun.

## 📄 Lisans (License)
Bu proje **MIT Lisansı** ile lisanslanmıştır. Daha fazla bilgi için `LICENSE` dosyasına göz atabilirsiniz.

---
*OxideNMS — Güvenli, Hızlı, Paslanmaz.* 🦀
