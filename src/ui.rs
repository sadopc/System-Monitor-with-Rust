// ui.rs - Terminal kullanıcı arayüzünü çizen modül
// Bu modül tıpkı bir grafik tasarımcı gibi, verileri görsel öğelere dönüştürür
use sysinfo::SystemExt;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Line,
    widgets::{
        Block, Borders, Chart, Dataset, Gauge, List, ListItem, 
        Paragraph, Sparkline, Table, Row, Cell
    },
    Frame,
};
use crate::app::App;

// Ana UI çizim fonksiyonu - her frame'de çağrılır
// Frame, ratatui'nin çizim yüzeyi - tıpkı ressamın tuvali gibi
// Not: Yeni API'de Frame artık generic parametre gerektirmez
pub fn ui(f: &mut Frame, app: &App) {
    // Terminal boyutunu al - responsive tasarım için gerekli
    let size = f.size();
    
    // Ana layout'u oluştur - tıpkı web tasarımında grid system gibi
    // Constraint::Percentage ile yüzdelik oranlar belirliyoruz
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // Üst başlık - 3 satır sabit
            Constraint::Min(10),       // Ana içerik - kalan alan
            Constraint::Length(3),     // Alt bilgi - 3 satır sabit
        ])
        .split(size);
    
    // Başlık bölümünü çiz
    draw_header(f, main_layout[0], app);
    
    // Ana içerik alanını yatay olarak böl
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Sol panel - CPU ve RAM
            Constraint::Percentage(40), // Sağ panel - Process listesi ve ağ
        ])
        .split(main_layout[1]);
    
    // Sol paneli dikey olarak böl
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // CPU bölümü
            Constraint::Percentage(50), // RAM bölümü
        ])
        .split(content_layout[0]);
    
    // CPU ve RAM bölümlerini çiz
    draw_cpu_section(f, left_layout[0], app);
    draw_memory_section(f, left_layout[1], app);
    
    // Sağ paneli dikey olarak böl
    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Process listesi
            Constraint::Percentage(40), // Ağ trafiği
        ])
        .split(content_layout[1]);
    
    // Process ve ağ bölümlerini çiz
    draw_process_section(f, right_layout[0], app);
    draw_network_section(f, right_layout[1], app);
    
    // Alt bilgi çubuğunu çiz
    draw_footer(f, main_layout[2]);
}

// Üst başlık bölümünü çizen fonksiyon
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    // Sistem uptime'ını formatla - saniyeden okunabilir formata
    // Modern API'de uptime() artık instance method
    let uptime = app.system.uptime();
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    let seconds = uptime % 60;
    
    // Başlık metnini oluştur - uygulamanın kimliği
    let header_text = format!(
        "🖥️  Rust System Monitor | Uptime: {:02}:{:02}:{:02} | CPU Cores: {} | Avg Usage: {:.1}%",
        hours, minutes, seconds,
        app.cpu_count(),
        app.cpu_average
    );
    
    // Paragraph widget'ı - metin göstermek için temel bileşen
    // Style ile renk ve formatı belirliyoruz
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        );
    
    f.render_widget(header, area);
}

// CPU bölümünü çizen fonksiyon - en karmaşık kısım
fn draw_cpu_section(f: &mut Frame, area: Rect, app: &App) {
    // CPU alanını yatay olarak böl
    let cpu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // CPU gauge'lar
            Constraint::Percentage(70), // CPU grafiği
        ])
        .split(area);
    
    // Sol taraf: Her çekirdek için gauge çiz
    draw_cpu_gauges(f, cpu_layout[0], app);
    
    // Sağ taraf: CPU kullanım grafiği
    draw_cpu_chart(f, cpu_layout[1], app);
}

// CPU gauge'larını çizen fonksiyon
fn draw_cpu_gauges(f: &mut Frame, area: Rect, app: &App) {
    let current_usage = app.current_cpu_usage();
    let cpu_count = current_usage.len();
    
    // Her çekirdek için bir satır ayırıyoruz
    // min(cpu_count, area_height - 2) ile sınırları kontrol ediyoruz
    let available_height = area.height.saturating_sub(2) as usize; // Border için 2 çıkar
    let visible_cpus = cpu_count.min(available_height);
    
    // Dinamik constraint'ler oluştur - çekirdek sayısına göre
    let constraints: Vec<Constraint> = (0..visible_cpus)
        .map(|_| Constraint::Length(1))
        .collect();
    
    if !constraints.is_empty() {
        let gauge_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(
                // İç alan - border'ları çıkardığımız kısım
                Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width.saturating_sub(2),
                    height: area.height.saturating_sub(2),
                }
            );
        
        // Her çekirdek için gauge çiz
        for (i, &usage) in current_usage.iter().take(visible_cpus).enumerate() {
            // Kullanım yüzdesine göre renk belirleme - görsel feedback
            let color = match usage as u8 {
                0..=50 => Color::Green,    // Düşük kullanım - yeşil
                51..=80 => Color::Yellow,  // Orta kullanım - sarı  
                81..=100 => Color::Red,    // Yüksek kullanım - kırmızı
                _ => Color::White,
            };
            
            // Gauge widget - progress bar benzeri
            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(color))
                .percent(usage as u16)
                .label(format!("CPU{}: {:.1}%", i, usage));
            
            f.render_widget(gauge, gauge_layout[i]);
        }
    }
    
    // Ana border'ı çiz
    let block = Block::default()
        .title("CPU Cores")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Blue));
    
    f.render_widget(block, area);
}

// CPU kullanım grafiğini çizen fonksiyon
fn draw_cpu_chart(f: &mut Frame, area: Rect, app: &App) {
    // Grafik için veri hazırlığı - zaman serisini koordinatlara dönüştür
    if app.cpu_history.is_empty() {
        // Veri yoksa boş grafik göster
        let block = Block::default()
            .title("CPU Usage History")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue));
        f.render_widget(block, area);
        return;
    }
    
    // Ortalama CPU kullanımı için dataset oluştur
    let cpu_data: Vec<(f64, f64)> = app.cpu_history
        .iter()
        .enumerate()
        .map(|(i, cpu_values)| {
            // Her zaman noktasında tüm çekirdeklerin ortalamasını al
            let avg = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;
            (i as f64, avg as f64)
        })
        .collect();
    
    // Grafik için x ve y eksen sınırlarını belirle
    let max_y = 100.0; // CPU yüzdesi max 100
    let max_x = app.cpu_history_len as f64;
    
    // Dataset oluştur - çizgiyi tanımlar
    // Modern ratatui'de marker için symbols modülünü kullanıyoruz
    let dataset = Dataset::default()
        .name("Avg CPU")
        .marker(symbols::Marker::Braille) // Braille karakterler ile yumuşak çizgi
        .style(Style::default().fg(Color::Cyan))
        .data(&cpu_data);
    
    // Chart widget'ı oluştur
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("CPU Usage History")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        )
        .x_axis(
            ratatui::widgets::Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_x])
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .title("Usage %")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_y])
        );
    
    f.render_widget(chart, area);
}

// RAM bölümünü çizen fonksiyon
fn draw_memory_section(f: &mut Frame, area: Rect, app: &App) {
    // RAM alanını yatay olarak böl
    let memory_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // RAM bilgileri
            Constraint::Percentage(50), // RAM grafiği
        ])
        .split(area);
    
    // Sol taraf: RAM bilgileri
    draw_memory_info(f, memory_layout[0], app);
    
    // Sağ taraf: RAM kullanım geçmişi
    draw_memory_chart(f, memory_layout[1], app);
}

// RAM bilgilerini gösteren fonksiyon
fn draw_memory_info(f: &mut Frame, area: Rect, app: &App) {
    let used_memory = app.system.used_memory();
    let total_memory = app.system.total_memory();
    let memory_percent = app.memory_usage_percent();
    
    // Swap bilgileri
    let used_swap = app.system.used_swap();
    let total_swap = app.system.total_swap();
    let swap_percent = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64 * 100.0) as f32
    } else {
        0.0
    };
    
    // RAM bilgilerini formatla
    let memory_text = format!(
        "RAM Usage: {:.1}%\n\
         Used: {}\n\
         Total: {}\n\
         Free: {}\n\
         \n\
         Swap Usage: {:.1}%\n\
         Used: {}\n\
         Total: {}",
        memory_percent,
        App::format_bytes(used_memory),
        App::format_bytes(total_memory),
        App::format_bytes(total_memory - used_memory),
        swap_percent,
        App::format_bytes(used_swap),
        App::format_bytes(total_swap)
    );
    
    let memory_info = Paragraph::new(memory_text)
        .block(
            Block::default()
                .title("Memory Info")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        )
        .style(Style::default().fg(Color::White));
    
    f.render_widget(memory_info, area);
}

// RAM kullanım grafiğini çizen fonksiyon
fn draw_memory_chart(f: &mut Frame, area: Rect, app: &App) {
    if app.memory_history.is_empty() {
        let block = Block::default()
            .title("Memory Usage History")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue));
        f.render_widget(block, area);
        return;
    }
    
    // RAM kullanım yüzdesini hesapla
    let memory_data: Vec<(f64, f64)> = app.memory_history
        .iter()
        .enumerate()
        .map(|(i, &(used, total))| {
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            (i as f64, percent)
        })
        .collect();
    
    let dataset = Dataset::default()
        .name("RAM")
        .marker(symbols::Marker::Braille) // Güncellenmiş symbol kullanımı
        .style(Style::default().fg(Color::Green))
        .data(&memory_data);
    
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("Memory Usage History")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        )
        .x_axis(
            ratatui::widgets::Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, app.cpu_history_len as f64])
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .title("Usage %")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0])
        );
    
    f.render_widget(chart, area);
}

// Process listesini çizen fonksiyon
fn draw_process_section(f: &mut Frame, area: Rect, app: &App) {
    let processes = app.top_processes();
    
    // Tablo başlıkları
    let header = Row::new(vec![
        Cell::from("Process"),
        Cell::from("CPU%"),
        Cell::from("Memory"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    // Process verilerini tablo satırlarına dönüştür
    let rows: Vec<Row> = processes
        .iter()
        .map(|(name, cpu, memory)| {
            Row::new(vec![
                Cell::from(name.clone()),
                Cell::from(format!("{:.1}", cpu)),
                Cell::from(App::format_bytes(*memory)),
            ])
        })
        .collect();
    
    // Kolon genişliklerini belirle
    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];
    
    // Modern ratatui API'sinde Table::new() artık widths parametresi de alır
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title("Top Processes")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        )
        .style(Style::default().fg(Color::White));
    
    f.render_widget(table, area);
}

// Ağ trafiği bölümünü çizen fonksiyon
fn draw_network_section(f: &mut Frame, area: Rect, app: &App) {
    // Son ağ verilerini al
    let (download_speed, upload_speed) = app.network_history
        .back()
        .copied()
        .unwrap_or((0, 0));
    
    let network_text = format!(
        "Network Traffic\n\
         \n\
         ⬇️ Download: {}/s\n\
         ⬆️ Upload: {}/s\n\
         \n\
         Press 'q' or ESC to quit",
        App::format_bytes(download_speed),
        App::format_bytes(upload_speed)
    );
    
    let network_info = Paragraph::new(network_text)
        .block(
            Block::default()
                .title("Network")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        )
        .style(Style::default().fg(Color::White));
    
    f.render_widget(network_info, area);
}

// Alt bilgi çubuğunu çizen fonksiyon
fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = "🦀 Built with Rust | Press 'q' or ESC to quit | Refresh Rate: 4 FPS";
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
        );
    
    f.render_widget(footer, area);
}
