use enigo::{Enigo, KeyboardControllable};

pub async fn type_text(text: &str, delay_ms: u32) {
    let text_clone = text.to_string();
    
    println!("Swift Speak: System typing started (Speed: {}ms) for '{}'", delay_ms, text_clone);
    let mut enigo = Enigo::new();
    
    // Wait for hotkey release and focus stability
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    if delay_ms == 0 {
        // Instant typing
        enigo.key_sequence(&text_clone);
    } else {
        // Per-character typing for better compatibility
        for c in text_clone.chars() {
            enigo.key_sequence(&c.to_string());
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
        }
    }
    
    println!("Swift Speak: Typing complete.");
}

pub async fn press_enter() {
    // Small delay to ensure focus and character stability
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    let mut enigo = Enigo::new();
    enigo.key_click(enigo::Key::Return);
    println!("Swift Speak: AI Mode - Enter pressed.");
}
