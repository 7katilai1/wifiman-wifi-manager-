# WifiMan 🛜

WifiMan is a fast, modern, and lightweight NetworkManager GUI utility written in **Rust** and **GTK4**. It serves as a direct drop-in replacement for `nmtui` and `nm-connection-editor`, built specifically with modern Wayland compositors (like Hyprland, Sway) and custom desktop setups in mind.

> 🤖 **Disclaimer:** This project was entirely generated with AI. Please note that while it offers many advanced network management features, **not all features have been extensively tested in real-world scenarios yet**. Use it with caution, expect potential bugs, and feel free to contribute or report issues!

## ✨ Features

- **Modern UI:** Clean GTK4 interface with dynamic signal strength indicators and network sorting.
- **Full Connection Management:** 
  - Scan and connect to Wi-Fi and Ethernet networks.
  - Create advanced profiles (Wi-Fi, Ethernet, Bond, Bridge, VLAN).
  - Modify existing profiles (Static IP, DHCP, DNS, Gateway, MTU, Cloned MAC).
  - Delete or forget saved networks.
- **System Settings:** Directly change your system's Hostname.
- **Context Menus:** Right-click context menus for quick actions.
- **Error Handling:** Built-in dialogs for user-friendly error reporting rather than terminal crashes.
- **No Daemon Required:** Interacts directly with `nmcli` in the background without needing a heavy daemon.

## 🚀 Installation

### Prerequisites
Make sure you have the following dependencies installed on your system:
- `rust`, `cargo`
- `gtk4` (development libraries)
- `NetworkManager` (`nmcli` must be installed)

### Building from Source

```bash
git clone https://github.com/7katilai1/wifiman-wifi-manager-.git
cd wifiman-wifi-manager-
cargo build --release
```

The compiled executable will be located in `target/release/wifi-manager`.

### Installing Locally
You can easily move the binary to your local bin directory to run it globally:

```bash
mkdir -p ~/.local/bin ~/.local/share/applications
cp target/release/wifi-manager ~/.local/bin/

# Optional: Add desktop entry
cp wifiman.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
```

## 🤝 Contributing
Pull requests are welcome! Feel free to open issues for bugs or feature requests.

## 📝 License
This project is licensed under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).

<img width="1334" height="854" alt="image" src="https://github.com/user-attachments/assets/9bd0708e-d05c-4b1b-8ac4-893270f38fbe" />


---

# WifiMan 🛜 (Türkçe)

WifiMan, **Rust** ve **GTK4** kullanılarak yazılmış hızlı, modern ve hafif bir NetworkManager GUI (Grafiksel Kullanıcı Arayüzü) aracıdır. Özellikle Hyprland, Sway gibi modern Wayland pencere yöneticileri düşünülerek tasarlanmış olup, `nmtui` ve `nm-connection-editor` araçlarının yerini doğrudan almak üzere geliştirilmiştir.

> 🤖 **Uyarı:** Bu proje tamamen yapay zeka ile oluşturulmuştur. Pek çok gelişmiş ağ yönetim özelliği sunmasına rağmen, **tüm özellikleri henüz gerçek dünya senaryolarında kapsamlı bir şekilde test edilmemiştir**. Lütfen dikkatli kullanın, olası hatalara karşı hazırlıklı olun ve katkıda bulunmaktan veya sorun bildirmekten çekinmeyin!

## ✨ Özellikler

- **Modern Arayüz:** Dinamik sinyal gücü göstergeleri ve ağ sıralama özelliklerine sahip temiz GTK4 arayüzü.
- **Tam Bağlantı Yönetimi:** 
  - Wi-Fi ve Ethernet ağlarını tarama ve bağlanma.
  - Gelişmiş profiller oluşturma (Wi-Fi, Ethernet, Bond, Bridge, VLAN).
  - Mevcut profilleri düzenleme (Statik IP, DHCP, DNS, Ağ Geçidi, MTU, Klonlanmış MAC).
  - Kaydedilmiş ağları silme veya sistemden unutma.
- **Sistem Ayarları:** Sisteminizin Hostname (Bilgisayar Adı) bilgisini doğrudan arayüzden değiştirme.
- **İçerik Menüleri (Context Menus):** Hızlı işlemler için ağların üzerine sağ tık menüleri.
- **Hata Yönetimi:** Terminalde uygulamanın çökmesi yerine kullanıcı dostu görsel hata iletişim kutuları (dialog).
- **Arka Plan Servisi (Daemon) Gerektirmez:** Arka planda doğrudan `nmcli` ile iletişim kurarak sistemi yormaz.

## 🚀 Kurulum

### Gereksinimler
Sisteminizde aşağıdaki bağımlılıkların kurulu olduğundan emin olun:
- `rust`, `cargo`
- `gtk4` (geliştirici paketleri)
- `NetworkManager` (`nmcli` komutu çalışıyor olmalıdır)

### Kaynak Koddan Derleme

```bash
git clone https://github.com/7katilai1/wifiman-wifi-manager-.git
cd wifiman-wifi-manager-
cargo build --release
```

Derlenen çalıştırılabilir dosya `target/release/wifi-manager` konumunda bulunacaktır.

### Lokele Kurulum
Uygulamayı işletim sisteminin herhangi bir yerinden (veya başlatıcıdan) kolayca çalıştırabilmek için dosyayı yerel klasörünüze taşıyabilirsiniz:

```bash
mkdir -p ~/.local/bin ~/.local/share/applications
cp target/release/wifi-manager ~/.local/bin/

# İsteğe bağlı: Uygulama menüsü kısayolu (Desktop entry) ekleme
cp wifiman.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
```

## 🤝 Katkıda Bulunma
Geliştirme talepleriniz (Pull request) her zaman kabul edilir! Hatalar (bug) veya yeni özellik istekleri için issue açmaktan çekinmeyin.

## 📝 Lisans
Bu proje [GNU Genel Kamu Lisansı v3.0 (GPL-3.0)](LICENSE) altında lisanslanmıştır.
