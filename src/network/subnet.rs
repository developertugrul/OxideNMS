//! Subnet (alt ag) hesaplamalari.
//!
//! Buradaki `Subnet` bir "nesne" gibi dusunulmeli: icinde bir IP ve bir
//! prefix tutar, geri kalan tum degerleri (network, broadcast, host araligi)
//! metotlarla hesaplar. Ekrana yazma veya girdi okuma YOKTUR — o isler
//! arayuz katmaninindir. Bu modul sadece "saf mantik".

use std::net::Ipv4Addr;

use crate::error::NetworkError;

/// Bir IPv4 subnet'ini temsil eder (orn: 192.168.1.10/24).
#[derive(Debug, Clone, Copy)]
pub struct Subnet {
    ip: Ipv4Addr,
    prefix: u8,
}

impl Subnet {
    /// "192.168.1.10/24" bicimindeki metinden bir `Subnet` uretir.
    ///
    /// Bu, OOP'deki "constructor" gibidir: gecerli olmayan girdiyi
    /// asla nesneye donusturmez, hata dondurur.
    pub fn parse(girdi: &str) -> Result<Self, NetworkError> {
        let (ip_kismi, prefix_kismi) =
            girdi.split_once('/').ok_or(NetworkError::InvalidFormat)?;

        let ip: Ipv4Addr = ip_kismi
            .parse()
            .map_err(|_| NetworkError::InvalidIp(ip_kismi.to_string()))?;

        let prefix: u8 = prefix_kismi
            .parse()
            .map_err(|_| NetworkError::InvalidPrefix(prefix_kismi.to_string()))?;

        Self::new(ip, prefix)
    }

    /// Dogrudan IP ve prefix ile bir `Subnet` uretir (dogrulama yapar).
    pub fn new(ip: Ipv4Addr, prefix: u8) -> Result<Self, NetworkError> {
        if prefix > 32 {
            return Err(NetworkError::PrefixOutOfRange(prefix));
        }
        Ok(Self { ip, prefix })
    }

    // --- Basit erisimciler (getter'lar) ---

    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    // --- Hesaplanan degerler ---

    /// Subnet maskesi (orn: /24 -> 255.255.255.0).
    pub fn mask(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.mask_u32())
    }

    /// Agin adresi: host bitleri sifirlanmis hali.
    pub fn network(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.ip) & self.mask_u32())
    }

    /// Broadcast adresi: host bitleri 1 yapilmis hali.
    pub fn broadcast(&self) -> Ipv4Addr {
        let network = u32::from(self.ip) & self.mask_u32();
        Ipv4Addr::from(network | !self.mask_u32())
    }

    /// Kullanilabilir ilk host. /31 ve /32'de klasik aralik olmadigi icin None.
    pub fn first_host(&self) -> Option<Ipv4Addr> {
        match self.prefix {
            32 => Some(self.ip),
            31 => Some(self.network()),
            _ => Some(Ipv4Addr::from(u32::from(self.network()) + 1)),
        }
    }

    /// Kullanilabilir son host.
    pub fn last_host(&self) -> Option<Ipv4Addr> {
        match self.prefix {
            32 => Some(self.ip),
            31 => Some(self.broadcast()),
            _ => Some(Ipv4Addr::from(u32::from(self.broadcast()) - 1)),
        }
    }

    /// Kullanilabilir host sayisi. (/0 icin deger buyuk oldugundan u64.)
    pub fn usable_hosts(&self) -> u64 {
        match self.prefix {
            32 => 1,
            31 => 2,
            _ => {
                let host_bit = 32 - self.prefix as u32;
                2u64.pow(host_bit) - 2
            }
        }
    }

    // --- Ic yardimci ---

    /// Maskeyi 32 bitlik sayi olarak uretir: soldan `prefix` kadar 1.
    fn mask_u32(&self) -> u32 {
        if self.prefix == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix)
        }
    }
}
