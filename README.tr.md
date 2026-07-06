# OxideNMS

OxideNMS, Cisco odakli ekipler icin gelistirilen profesyonel bir masaustu ag
yonetimi ve siber guvenlik operasyon uygulamasidir.

Uygulama; CCNA, CCNP, CCIE, NOC, NetOps ve guvenlik operasyon kullanicilarinin
gunluk Cisco ag operasyonlarini hizli, yerel ve kontrollu sekilde yonetebilmesi
icin tasarlanmistir.

## OxideNMS Ne Yapar?

OxideNMS, ag konfigurasyon yonetiminde ihtiyac duyulan temel is akislarini tek
masaustu uygulamasinda toplar:

- Cihaz envanteri ve yasam dongusu takibi.
- Sifreli yerel credential vault.
- SSH ile running-config alma.
- Konfigurasyon yedek gecmisi.
- Konfigurasyon karsilastirma ve degisiklik inceleme.
- Cisco konfigurasyon hardening denetimleri.
- Dry-run ve onay akisi olan toplu komut dagitimi.
- Operasyon audit log kaydi.
- Cihaz kesif taramasi.
- Canli gorunurluk icin syslog dinleyici.
- SNMP topoloji/durum haritasi temeli.
- VLAN, subnet, template ve firmware operasyon ekranlari.
- Resmi release manifest uzerinden zorunlu guncelleme.

## Kimler Icin?

OxideNMS, Cisco ortamlarini yoneten muhendis ve operasyon ekipleri icin
gelistirilmektedir:

- CCNA, CCNP veya CCIE seviyesindeki ag uzmanlari.
- Operasyonel tutarliliktan sorumlu NOC ve NetOps ekipleri.
- Ag cihazi guvenlik durusunu inceleyen siber guvenlik ekipleri.
- Kontrollu ve tasinabilir Cisco operasyon calisma istasyonu isteyen
  danismanlar.

## Guvenlik Modeli

OxideNMS operasyon verisini yerel olarak saklar ve cihaz kimlik bilgilerini
master password ile korur. Vault baslatma, cihaz degisiklikleri, yedekleme,
kesif importlari ve toplu komut operasyonlari gibi hassas aksiyonlar audit log
olarak kaydedilir.

Guncelleme kontrolu zorunludur. Resmi yeni surum yayinlandiginda eski surumler
uygulamayi kilitler ve kullanici guncel surumu kurmadan devam edemez.
Guncelleme manifest adresi uygulama icinde sabittir; son kullanici tarafindan
degistirilemez.

## Kurulum

Windows kullanicilari OxideNMS'i GitHub Releases sayfasindaki resmi kurulum
paketiyle kurmalidir:

[Son OxideNMS surumunu indir](https://github.com/developertugrul/OxideNMS/releases/latest)

Windows installer, OxideNMS'i Program Files altina kurar, Start Menu kisayolu
olusturur, kaldirma kaydi ekler ve istege bagli masaustu kisayolu olusturabilir.

Linux ve macOS release assetleri su anda uygulama binary dosyasi olarak
dagitilmaktadir.

## Urun Yonu

OxideNMS profesyonel NMS/NCCCM is akislarina dogru gelistirilmektedir:

- Zamanlanmis konfigurasyon yedegi ve saklama politikasi.
- Degisiklik takibi ve rollback hazirligi.
- Compliance policy setleri ve disa aktarilabilir raporlar.
- SNMPv3 credential profilleri.
- CDP/LLDP topoloji zenginlestirme.
- Toplu operasyonlar icin daha guvenli job queue yapisi.
- Security posture dashboard ve alarm mekanizmalari.

## Dokumantasyon

- [Yol haritasi](docs/ROADMAP.md)
- [Release sureci](docs/RELEASE.md)
- [Guvenlik politikasi](SECURITY.md)
- [Katki rehberi](CONTRIBUTING.md)
- [Degisiklik gunlugu](CHANGELOG.md)

## Lisans

OxideNMS MIT Lisansi ile lisanslanmistir. Detaylar icin [LICENSE](LICENSE).
