// system_info.rs - Gelişmiş sistem bilgisi toplama modülü
// Bu modül gelecekteki genişlemeler için hazırlanmış bir temel sağlar
// Örneğin: sıcaklık sensörleri, disk bilgileri, GPU kullanımı gibi

use sysinfo::{System, SystemExt, DiskExt, ComponentExt};

// Disk kullanım bilgilerini tutan struct
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,           // Disk adı (örn: "/dev/sda1")
    pub mount_point: String,    // Bağlanma noktası (örn: "/", "/home")
    pub total_space: u64,       // Toplam alan (byte)
    pub available_space: u64,   // Kullanılabilir alan (byte)
    pub used_space: u64,        // Kullanılan alan (byte)
    pub usage_percent: f32,     // Kullanım yüzdesi
    pub file_system: String,    // Dosya sistemi türü (ext4, ntfs, vs.)
}

// Sistem sıcaklık bilgilerini tutan struct
#[derive(Debug, Clone)]
pub struct TemperatureInfo {
    pub component_name: String, // Bileşen adı (CPU, GPU, vs.)
    pub current_temp: f32,      // Şu anki sıcaklık (Celsius)
    pub max_temp: Option<f32>,  // Maksimum sıcaklık (varsa)
    pub critical_temp: Option<f32>, // Kritik sıcaklık (varsa)
}

// Gelişmiş sistem bilgileri için ana struct
pub struct SystemInfoCollector {
    system: System,
}

impl SystemInfoCollector {
    // Yeni bir collector oluştur
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
    
    // Sistem verilerini yenile - her güncelleme öncesi çağrılmalı
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    // Tüm disk bilgilerini topla
    // Bu fonksiyon sistem üzerindeki tüm bağlı diskleri tarar
    // Modern sysinfo API'sinde disks() artık System'da instance method
    pub fn get_disk_info(&self) -> Vec<DiskInfo> {
        self.system
            .disks()
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total - available;
                
                // Kullanım yüzdesini hesapla - sıfıra bölme kontrolü önemli
                let usage_percent = if total > 0 {
                    (used as f64 / total as f64 * 100.0) as f32
                } else {
                    0.0
                };
                
                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    usage_percent,
                    file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
                }
            })
            .collect()
    }
    
    // Sistem sıcaklık bilgilerini topla
    // Bu özellik her sistemde mevcut olmayabilir - özellikle sanal makinelerde
    // Modern API'de components() method'u da değişmemiş
    pub fn get_temperature_info(&self) -> Vec<TemperatureInfo> {
        self.system
            .components()
            .iter()
            .map(|component| {
                let max = component.max();
                let critical = component.critical();
                TemperatureInfo {
                    component_name: component.label().to_string(),
                    current_temp: component.temperature(),
                    max_temp: (max > 0.0).then(|| max),
                    critical_temp: critical.filter(|&c| c > 0.0),
                }
            })
            .collect()
    }
    
    // Sistem boot zamanını al
    // Modern API'de artık instance method
    pub fn get_boot_time(&self) -> u64 {
        self.system.boot_time()
    }
    
    // Toplam process sayısını al
    pub fn get_process_count(&self) -> usize {
        self.system.processes().len()
    }
    
    // Sistem hostname'ini al
    // Modern API'de artık instance method
    pub fn get_hostname(&self) -> Option<String> {
        self.system.host_name()
    }
    
    // Sistem çekirdek versiyonunu al
    // Modern API'de artık instance method
    pub fn get_kernel_version(&self) -> Option<String> {
        self.system.kernel_version()
    }
    
    // İşletim sistemi bilgilerini al
    // Modern API'de artık instance method'lar
    pub fn get_os_info(&self) -> (Option<String>, Option<String>) {
        (
            self.system.name(),           // OS adı (Linux, Windows, macOS)
            self.system.os_version()      // OS versiyonu
        )
    }
    
    // CPU fiziksel çekirdek sayısı - hyperthreading dikkate alınmaz
    pub fn get_physical_core_count(&self) -> Option<usize> {
        self.system.physical_core_count()
    }
    
    // Sistem load average (sadece Unix/Linux sistemlerde)
    // 1, 5 ve 15 dakikalık ortalama yükü gösterir
    #[cfg(target_family = "unix")]
    pub fn get_load_average(&self) -> Option<(f64, f64, f64)> {
        // Load average bilgisini almak için sysinfo'nun kısıtlamaları var
        // Gelecek versiyonlarda bu özellik eklenebilir
        // Şimdilik None döndürüyoruz
        None
    }
    
    // Windows sistemler için - sadece placeholder
    #[cfg(target_family = "windows")]
    pub fn get_load_average(&self) -> Option<(f64, f64, f64)> {
        // Windows'ta load average konsepti yoktur
        None
    }
}

// Yardımcı fonksiyonlar - UI tarafından kullanılabilir

// Sıcaklık verilerini kategorize et - kritik sıcaklıkları belirle
pub fn categorize_temperature(temp: f32) -> TemperatureCategory {
    match temp as u32 {
        0..=40 => TemperatureCategory::Cool,      // Soğuk - yeşil
        41..=60 => TemperatureCategory::Normal,   // Normal - mavi
        61..=75 => TemperatureCategory::Warm,     // Ilık - sarı
        76..=85 => TemperatureCategory::Hot,      // Sıcak - turuncu
        86.. => TemperatureCategory::Critical,    // Kritik - kırmızı
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemperatureCategory {
    Cool,
    Normal,
    Warm,
    Hot,
    Critical,
}

// Disk kullanımını kategorize et - renk kodlaması için
pub fn categorize_disk_usage(usage_percent: f32) -> DiskUsageCategory {
    match usage_percent as u32 {
        0..=70 => DiskUsageCategory::Normal,      // Normal kullanım - yeşil
        71..=85 => DiskUsageCategory::Warning,    // Uyarı - sarı
        86..=95 => DiskUsageCategory::Critical,   // Kritik - turuncu
        96.. => DiskUsageCategory::Full,          // Dolu - kırmızı
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiskUsageCategory {
    Normal,
    Warning,
    Critical,
    Full,
}

// Byte'ları insan tarafından okunabilir formata çevir
// Bu fonksiyon App struct'ındaki ile aynı - gelecekte tek yerde toplanabilir
pub fn format_bytes_detailed(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    // Hassasiyet - büyük dosyalar için daha az ondalık
    let precision = match unit_index {
        0..=1 => 0,  // Byte ve KB için tam sayı
        2 => 1,      // MB için 1 ondalık
        _ => 2,      // GB ve üzeri için 2 ondalık
    };
    
    format!("{:.precision$} {}", size, UNITS[unit_index], precision = precision)
}

// Uptime'ı detaylı formata çevir
pub fn format_uptime(uptime_seconds: u64) -> String {
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

// Test fonksiyonları - gelişim aşamasında kullanışlı
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temperature_categorization() {
        assert_eq!(categorize_temperature(30.0), TemperatureCategory::Cool);
        assert_eq!(categorize_temperature(50.0), TemperatureCategory::Normal);
        assert_eq!(categorize_temperature(70.0), TemperatureCategory::Warm);
        assert_eq!(categorize_temperature(80.0), TemperatureCategory::Hot);
        assert_eq!(categorize_temperature(90.0), TemperatureCategory::Critical);
    }
    
    #[test]
    fn test_disk_usage_categorization() {
        assert_eq!(categorize_disk_usage(50.0), DiskUsageCategory::Normal);
        assert_eq!(categorize_disk_usage(80.0), DiskUsageCategory::Warning);
        assert_eq!(categorize_disk_usage(90.0), DiskUsageCategory::Critical);
        assert_eq!(categorize_disk_usage(98.0), DiskUsageCategory::Full);
    }
    
    #[test]
    fn test_byte_formatting() {
        assert_eq!(format_bytes_detailed(1024), "1 KB");
        assert_eq!(format_bytes_detailed(1536), "1.5 KB");
        assert_eq!(format_bytes_detailed(1073741824), "1.00 GB");
    }
    
    #[test]
    fn test_uptime_formatting() {
        assert_eq!(format_uptime(30), "30s");
        assert_eq!(format_uptime(3661), "1h 1m 1s");
        assert_eq!(format_uptime(90061), "1d 1h 1m 1s");
    }
}