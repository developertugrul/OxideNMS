//! Komut satiri arayuzu (CLI).
//!
//! Bu, projenin ilk "arayuz" katmani. Gorevi:
//!   1. Kullanicidan komut/arguman almak (clap ile),
//!   2. Uygun domain fonksiyonunu cagirmak (network::Subnet),
//!   3. Sonucu ekrana basmak.
//!
//! Domain mantigi burada YOK; burasi sadece "koprü".

use clap::{Parser, Subcommand};

use crate::network::Subnet;

/// Cisco ag araclari — komut satiri arayuzu.
#[derive(Parser)]
#[command(name = "cisco", version, about = "Cisco ag araclari (Rust)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Desteklenen alt komutlar. Her yeni ozellik buraya bir komut olarak eklenir.
#[derive(Subcommand)]
pub enum Commands {
    /// Bir CIDR icin subnet bilgisini hesaplar (orn: 192.168.1.10/24).
    Subnet {
        /// ip/prefix biciminde adres, orn: 192.168.1.10/24
        cidr: String,
    },
}

/// Uygulamanin giris akisi: komutu ayristir ve calistir.
pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Subnet { cidr } => subnet_komutu(&cidr),
    }
}

/// `subnet` komutunun govdesi.
fn subnet_komutu(cidr: &str) {
    match Subnet::parse(cidr) {
        Ok(net) => yazdir(&net),
        Err(hata) => {
            // Hatayi kullaniciya goster ve hata koduyla cik.
            eprintln!("Hata: {hata}");
            std::process::exit(1);
        }
    }
}

/// Bir subnet'in tum bilgilerini duzenli bicimde ekrana basar.
fn yazdir(net: &Subnet) {
    println!("Input        : {}/{}", net.ip(), net.prefix());
    println!("Subnet maske : {}", net.mask());
    println!("Network      : {}", net.network());
    println!("Broadcast    : {}", net.broadcast());

    match (net.first_host(), net.last_host()) {
        (Some(ilk), Some(son)) => {
            println!("Ilk host     : {ilk}");
            println!("Son host     : {son}");
        }
        _ => println!("Ilk/Son host : (bu prefix icin kullanilabilir aralik yok)"),
    }

    println!("Host sayisi  : {}", net.usable_hosts());
}
