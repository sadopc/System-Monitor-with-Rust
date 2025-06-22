// Ana program dosyası - uygulamamızın giriş noktası
// Bu dosya tıpkı bir evin kapısı gibi, tüm bileşenleri bir araya getirir

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

// Kendi modüllerimizi import ediyoruz
mod app;           // Uygulamanın ana mantığı burada olacak
mod ui;            // Kullanıcı arayüzü komponetleri
mod system_info;   // Sistem bilgilerini toplayan modül

use app::App;
use ui::ui;

// Ana async fonksiyon - Rust'ta async main için tokio macro kullanılır
#[tokio::main]
async fn main() -> Result<()> {
    // Terminal'i ham moda alıyoruz - bu sayede karakterleri tek tek yakalayabiliriz
    // Tıpkı bir piyanist gibi her tuşa ayrı ayrı tepki verebileceğiz
    enable_raw_mode()?;
    
    let mut stdout = io::stdout();
    // Alternatif ekrana geçiyoruz - bu sayede mevcut terminal içeriğini bozmayız
    // Uygulama kapandığında eski ekran geri gelecek
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Terminal backend'ini kuruyoruz - ratatui'nin crossterm ile konuşması için köprü
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Uygulamamızın ana durumunu tutacak struct'ı oluşturuyoruz
    let mut app = App::new().await?;
    
    // Ana event loop - tüm modern GUI uygulamalarında böyle bir döngü vardır
    // Event gelir → İşlenir → UI güncellenir → Tekrar event beklenir
    let tick_rate = Duration::from_millis(250); // 4 FPS - sistem bilgilerini güncellemek için
    let mut last_tick = Instant::now();
    let tick_delay = tokio::time::Duration::from_millis(500);
    // Update network calculation in app.rs:
    let time_delta = 0.5; // Instead of 0.25
    loop {
        // UI'yi çiziyoruz - her frame'de ekranı yeniden çizer
        terminal.draw(|f| ui(f, &app))?;

        // Event handling - kullanıcı girişini kontrol ediyoruz
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Sadece key press olaylarını işliyoruz (key release değil)
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break, // 'q' tuşuna basınca çık
                        KeyCode::Esc => break,       // Escape tuşuna basınca çık
                        _ => {} // Diğer tuşları şimdilik görmezden gel
                    }
                }
            }
        }

        // Belirli aralıklarla sistem bilgilerini güncelle
        if last_tick.elapsed() >= tick_rate {
            app.update().await?;
            last_tick = Instant::now();
        }
    }

    // Temizlik işlemleri - uygulamadan çıkarken terminal'i eski haline döndür
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
