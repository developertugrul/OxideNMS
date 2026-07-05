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
    <b>English</b> | <a href="README.tr.md">Türkçe</a>
  </p>
</div>

## 📌 About
**OxideNMS** is a high-performance, enterprise-grade Network Management System (NMS) developed for network administrators and engineers. Powered by the **Rust** programming language and the **egui** graphics library, it comes as a single executable and launches in milliseconds. Securely manage, backup, monitor, and template your network infrastructure.

---

## 🚀 Features

### 📊 Monitoring & Management
- **Performance Dashboard:** Monitor CPU and RAM usage of your devices with real-time graphs.
- **SNMP Topology Map (Cisco CDP/LLDP):** Discover devices on your network via SNMP and draw connection maps.
- **Built-in Syslog Server (UDP 514):** Centrally collect and analyze logs from your network devices.

### ⚙️ Configuration & Automation
- **Bulk Deploy:** Send configurations to hundreds of devices with a single click.
- **Template Engine (Jinja2):** Create parametric network configuration templates powered by the `minijinja` engine.
- **Auto Backup Service:** Automatically fetch and archive network device configurations in the background.
- **Diff Engine:** Compare old and new configurations of devices side-by-side (Git diff style).
- **Auto Firmware (IOS) Update:** Easily update device images via integrated TFTP/FTP workflows.

### 🛠️ Tools & Utilities
- **Device Manager:** Keep track of IP addresses, credentials, and hardware inventory for all your network equipment (SQLite).
- **Subnet Calculator (IPv4/IPv6):** Fast and practical IP subnetting.
- **VLAN Manager:** Easy VLAN pool management for switches.
- **Built-in SSH Terminal:** Connect directly to devices in seconds without needing an external client like PuTTY or SecureCRT.

### 🔒 Enterprise Security
- **Master Password Encryption (AES-256-GCM):** Passwords in the database are encrypted using your master password. Device credentials are never stored as plaintext on disk!

---

## 📥 Installation

OxideNMS is distributed as a single executable file. You do not need to install any heavy dependencies (like Python, Java, etc.).

### Build from Source
If you have [Rust](https://rustup.rs/) installed:

```bash
# Clone the repository
git clone https://github.com/developertugrul/OxideNMS.git
cd OxideNMS

# Build and run
cargo run --release
```
Once compiled, you can move the generated `target/release/oxidenms.exe` file anywhere and run it independently.

---

## 📚 Usage
1. **First Launch:** When you open the application, you will be prompted to set/enter a **Master Password**. This password is used to securely encrypt your device credentials in the database.
2. **Adding Devices:** Go to the *Device Manager* from the left menu and add your devices.
3. **Management:** Select the tool you need and start managing your network!

---

## 👨‍💻 Contributing
This project is **Open Source** and welcomes contributions. Feel free to submit PRs and open Issues!

1. Fork the project.
2. Create your feature branch (`git checkout -b feature/AmazingFeature`).
3. Commit your changes (`git commit -m 'feat: Add some AmazingFeature'`).
4. Push to the branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.

---

## 📜 License
This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for more details.
