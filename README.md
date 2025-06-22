# Rust System Monitor

🎯 **Rust ile yazılmış hafif, hızlı ve terminal tabanlı sistem izleyici.**

Bu proje, sisteminizdeki CPU, bellek, disk ve işlem bilgilerini canlı olarak gösteren modern bir terminal kullanıcı arayüzü sunar.

![resim](https://github.com/user-attachments/assets/e59b20a3-55d4-4cfa-ae3d-572314b05fdd)


## ✨ Özellikler

- Gerçek zamanlı sistem bilgisi takibi:
  - CPU çekirdek kullanımı
  - RAM ve swap kullanımı
  - Disk bilgileri
  - Uptime ve işlem sayısı
- Çok çekirdekli işlemci desteği
- `crossterm` tabanlı tuş yakalama
- Sekmeli (tab) arayüz
- Minimalist ve okunabilir tasarım
- Yüksek performanslı ve asenkron yapı (`tokio`)

## 📦 Bağımlılıklar

| Crate      | Açıklama |
|------------|----------|
| `sysinfo`  | Sistem bilgilerini toplar (CPU, RAM, Disk, vs.) |
| `ratatui`  | Terminal UI oluşturur (widget'lar, grafikler) |
| `crossterm`| Terminal girişi ve kontrolü |
| `tokio`    | Asenkron görevler ve zamanlayıcı |
| `anyhow`   | Hata yönetimi kolaylaştırması için |
| `chrono`   | Tarih/saat işlemleri için |

## 🚀 Kurulum ve Çalıştırma

### 1. Depoyu klonla

```bash
git clone https://github.com/sadopc/System-Monitor-with-Rust
cd rust-system-monitor
