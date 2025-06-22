// app.rs - Uygulamanın ana state management'ını yapan modül
// Bu dosya tıpkı bir yöneticinin rolünü oynar - tüm bilgileri toplar, düzenler ve sunar

use anyhow::Result;
use sysinfo::{System, SystemExt, CpuExt, NetworkExt, ProcessExt};
use std::collections::VecDeque;

// Uygulamamızın tüm durumunu tutan ana struct
// Rust'ta struct'lar hem veri hem de davranış (method) barındırabilir
pub struct App {
    // Sistem bilgilerini almak için sysinfo'nun System struct'ını kullanacağız
    pub system: System,
    
    // CPU kullanım geçmişini tutmak için - grafikler çizmek için gerekli
    // VecDeque bir çift yönlü kuyruk, hem baştan hem sondan eleman ekleyip çıkarabiliriz
    pub cpu_history: VecDeque<Vec<f32>>, // Her indeks bir çekirdek, değer kullanım yüzdesi
    pub cpu_history_len: usize,          // Kaç saniye geçmiş tutacağımız
    
    // RAM kullanımı için geçmiş verileri
    pub memory_history: VecDeque<(u64, u64)>, // (kullanılan, toplam) formatında
    
    // Ağ trafiği için - indirme ve yükleme hızlarını izlemek
    pub network_history: VecDeque<(u64, u64)>, // (indirme, yükleme) byte/s
    
    // Önceki ağ verilerini tutuyoruz - hız hesaplamak için fark almamız gerekir
    pub prev_network_data: Option<(u64, u64)>,
    
    // CPU kullanımının moving average'ı - anlık dalgalanmaları yumuşatmak için
    pub cpu_average: f32,
    pub cpu_scroll: usize, // yeni
}

impl App {
    // Constructor - yeni bir App instance'ı oluşturur
    // async çünkü sistem bilgilerini ilk kez toplarken zaman alabilir
    pub async fn new() -> Result<Self> {
        let mut system = System::new_all();
        
        // İlk refresh - sistem bilgilerini doldurmak için
        // System::new_all() boş bir sistem oluşturur, refresh ile doldururuz
        system.refresh_all();
        
        // CPU çekirdek sayısını öğreniyoruz - dinamik olarak array boyutu belirleme
        let cpu_count = system.cpus().len();
        
        // Geçmiş için 60 saniye tutacağız (4 FPS * 60 = 240 entry)
        let history_len = 60 * 4;
        
        // Her CPU çekirdeği için başlangıçta 0.0 değeri
        let initial_cpu_data = vec![0.0; cpu_count];
        
        let mut app = App {
            system,
            cpu_history: VecDeque::with_capacity(history_len),
            cpu_history_len: history_len,
            memory_history: VecDeque::with_capacity(history_len),
            network_history: VecDeque::with_capacity(history_len),
            prev_network_data: None,
            cpu_average: 0.0,
            cpu_scroll: 0, // yeni
        };
        
        // İlk CPU verilerini kuyruğa ekle
        app.cpu_history.push_back(initial_cpu_data);
        
        Ok(app)
    }
    
    // Sistem bilgilerini güncelleyen method - her frame'de çağrılacak
    pub async fn update(&mut self) -> Result<()> {
        // Sistem verilerini yenile - bu CPU, RAM, disk, ağ bilgilerini günceller
        self.system.refresh_all();
        
        // CPU bilgilerini güncelle
        self.update_cpu_data();
        
        // RAM bilgilerini güncelle  
        self.update_memory_data();
        
        // Ağ bilgilerini güncelle
        self.update_network_data();
        
        Ok(())
    }
    
    // CPU verilerini güncelleyen private method
    fn update_cpu_data(&mut self) {
        // Her CPU çekirdeğinin kullanımını bir vector'e topluyoruz
        let cpu_usage: Vec<f32> = self.system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage()) // Her çekirdeğin kullanım yüzdesini al
            .collect();
        
        // Geçmiş verilerimize yeni veriyi ekliyoruz
        self.cpu_history.push_back(cpu_usage.clone());
        
        // Eğer belirlediğimiz limiti aştıysak en eski veriyi çıkar
        // Bu sayede sabit boyutlu bir sliding window elde ederiz
        if self.cpu_history.len() > self.cpu_history_len {
            self.cpu_history.pop_front();
        }
        
        // Ortalama CPU kullanımını hesapla - tüm çekirdeklerin ortalaması
        // iter() → sum() → fold işlemi functional programming yaklaşımı
        self.cpu_average = cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32;
    }
    
    // RAM verilerini güncelleyen method
    fn update_memory_data(&mut self) {
        let used_memory = self.system.used_memory();
        let total_memory = self.system.total_memory();
        
        // Memory verilerini geçmişe ekle
        self.memory_history.push_back((used_memory, total_memory));
        
        // Sliding window mantığı - burada da aynı stratejiyi uyguluyoruz
        if self.memory_history.len() > self.cpu_history_len {
            self.memory_history.pop_front();
        }
    }
    
    // Ağ trafiği verilerini güncelleyen method
    fn update_network_data(&mut self) {
        // Modern sysinfo API'sinde networks() artık System üzerinde direkt method
        // Tüm ağ interface'lerinin verilerini topluyoruz
        let mut total_received = 0;
        let mut total_transmitted = 0;
        
        // self.system.networks() tüm ağ arayüzlerini döndürür (eth0, wlan0, vs.)
        // Yeni API'de Networks struct'ı üzerinden iterate ediyoruz
        for (_interface_name, network) in self.system.networks() {
            total_received += network.received();
            total_transmitted += network.transmitted();
        }
        
        // Eğer önceki veri varsa, hız hesaplayabiliriz
        if let Some((prev_received, prev_transmitted)) = self.prev_network_data {
            // Saniye başına byte hesaplama - delta / time
            // Burada time = 0.25 saniye (çünkü 4 FPS ile güncelliyoruz)
            let download_speed = ((total_received.saturating_sub(prev_received) as f64) / 0.25) as u64;
            let upload_speed = ((total_transmitted.saturating_sub(prev_transmitted) as f64) / 0.25) as u64;
            
            self.network_history.push_back((download_speed, upload_speed));
            
            // Sliding window
            if self.network_history.len() > self.cpu_history_len {
                self.network_history.pop_front();
            }
        }
        
        // Şu anki veriyi bir sonraki hesaplama için saklıyoruz
        self.prev_network_data = Some((total_received, total_transmitted));
    }
    
    // UI'nin kullanabileceği yardımcı method'lar
    
    // Toplam CPU çekirdek sayısını döndür
    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }
    
    // En son CPU verilerini döndür - UI'de anlık değerleri göstermek için
    pub fn current_cpu_usage(&self) -> Vec<f32> {
        self.cpu_history
            .back() // En son eklenen veri
            .cloned() // Ownership transferi için klon
            .unwrap_or_default() // Eğer veri yoksa boş vector döndür
    }
    
    // RAM kullanım yüzdesini hesapla
    pub fn memory_usage_percent(&self) -> f32 {
        let used = self.system.used_memory() as f64;
        let total = self.system.total_memory() as f64;
        
        if total > 0.0 {
            ((used / total) * 100.0) as f32
        } else {
            0.0
        }
    }
    
    // İnsan tarafından okunabilir boyut formatı (KB, MB, GB)
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        // 1024'e bölerek hangi birime yaklaştığımızı buluyoruz
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1} {}", size, UNITS[unit_index])
    }
    
    // En çok CPU kullanan processler - performans analizi için
    pub fn top_processes(&self) -> Vec<(String, f32, u64)> {
        let mut processes: Vec<_> = self.system
            .processes()
            .values()
            .map(|p| (
                p.name().to_string(),           // Process adı
                p.cpu_usage(),                  // CPU kullanımı
                p.memory()                      // RAM kullanımı
            ))
            .collect();
        
        // CPU kullanımına göre sırala (yüksekten alçağa)
        processes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // İlk 10 process'i döndür
        processes.into_iter().take(10).collect()
    }
}