# OxideNMS

OxideNMS, Cisco odaklı ağ yönetimi, konfigürasyon kontrolü ve siber güvenlik
operasyonları için geliştirilen Rust tabanlı masaüstü uygulamasıdır.

Hedef kullanıcılar CCNA, CCNP ve CCIE seviyesindeki ağ uzmanları ve güvenlik
operatörleridir. Amaç; cihaz envanteri, konfigürasyon yedeği, diff, hardening
denetimi, syslog görünürlüğü, SNMP kontrolleri ve kontrollü operasyon akışları
için hızlı ve yerel çalışan profesyonel bir araç sunmaktır.

## Mevcut Yetenekler

- Yerel SQLite tabanlı cihaz envanteri.
- Master password ile AES-256-GCM kimlik bilgisi şifreleme.
- SSH ile running-config alma.
- Konfigürasyon geçmişi ve yan yana diff akışı.
- Cisco konfigürasyon hardening denetim kuralları.
- Toplu komut gönderme iş akışı.
- Otomatik yedekleme worker temeli.
- SNMP durum haritası temeli.
- Dahili UDP syslog dinleyici.
- VLAN, subnet, template, firmware ve yardımcı araç ekranları.
- Gömülü fontlarla çoklu dil UI altyapısı.
- Zorunlu güncelleme manifest desteği.

Bazı ekranlar hâlâ temel seviyededir ve yol haritasında izlenmektedir.
OxideNMS profesyonel NMS/NCCCM kullanımına doğru olgunlaştırılır.

## Ürün Yönü

OxideNMS şu profesyonel ağ konfigürasyon yönetimi beklentilerine göre
geliştirilecektir:

- Keşif ve cihaz yaşam döngüsü envanteri.
- Otomatik konfigürasyon yedekleme ve saklama politikası.
- Değişiklik takibi, diff ve rollback hazırlığı.
- Compliance ve siber güvenlik duruş raporları.
- SNMP, CDP ve LLDP ile topoloji görünürlüğü.
- Preview, onay ve audit log içeren güvenli toplu operasyonlar.
- GitHub Release tabanlı masaüstü dağıtımı.

## Derleme ve Çalıştırma

Rust kurulu değilse <https://rustup.rs/> üzerinden kurun:

```bash
cargo run --release
```

Değişiklik yayınlamadan önce:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## Release

GitHub Release, sürüm tag'i ile oluşturulur:

```bash
git tag v1.0.2
git push origin v1.0.2
```

Workflow Windows, Linux ve macOS binary üretir. Artifact isimleri
`OxideNMS-{platform}-{arch}` standardını kullanır.

## Güncelleme Manifesti

OxideNMS sabit bir JSON manifest okur. `latest_version` çalışan sürümden
yeniyse uygulama güncellenene kadar kilitlenir. Manifest URL'i kullanıcı
tarafından değiştirilemez. Örnek dosya: [assets/latest.example.json](assets/latest.example.json).

```json
{
  "latest_version": "1.0.2",
  "minimum_version": "1.0.2",
  "download_url": "https://github.com/developertugrul/OxideNMS/releases/latest",
  "notes": "Güvenlik ve güvenilirlik sürümü."
}
```

`latest_version` zorunlu güncelleme kilidini belirler. `minimum_version`
uyumluluk için manifestte kalır ve zorunlu taban sürümle aynı tutulmalıdır.

## Dokümantasyon

- [Yol haritası](docs/ROADMAP.md)
- [Release süreci](docs/RELEASE.md)
- [Güvenlik politikası](SECURITY.md)
- [Katkı rehberi](CONTRIBUTING.md)
- [Değişiklik günlüğü](CHANGELOG.md)

## Lisans

OxideNMS MIT Lisansı ile lisanslanmıştır. Detaylar için [LICENSE](LICENSE).
