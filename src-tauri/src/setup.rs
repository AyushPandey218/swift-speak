use tauri::{AppHandle, Emitter, Manager};
use std::fs::File;
use std::io::copy;

pub async fn download_engine(app: AppHandle) -> Result<(), String> {
    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Downloading Model..." })).unwrap();

    let client = reqwest::Client::new();
    let model_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";
    
    let dest_dir = app.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&dest_dir).unwrap();
    
    let model_path = dest_dir.join("ggml-base.bin");
    
    if !model_path.exists() {
        let response = client.get(model_url).send().await.map_err(|e| e.to_string())?;
        let content = response.bytes().await.map_err(|e| e.to_string())?;
        let mut file = File::create(&model_path).map_err(|e| e.to_string())?;
        copy(&mut content.as_ref(), &mut file).map_err(|e| e.to_string())?;
    }

    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Model Ready. Place whisper-main.exe in AppData." })).unwrap();
    
    // Provide the path to the user in the console
    println!("App Data Directory: {:?}", dest_dir);
    
    Ok(())
}

pub fn position_overlay(window: &tauri::WebviewWindow, position: &str) -> Result<(), String> {
    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();
        let window_size = window.outer_size().map_err(|e| e.to_string())?;
        
        let (x, y) = match position {
            "top-left" => (monitor_pos.x + 20, monitor_pos.y + 20),
            "top-center" => (monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2, monitor_pos.y + 20),
            "top-right" => (monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - 20, monitor_pos.y + 20),
            "left" => (monitor_pos.x + 20, monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2),
            "center" => {
                let _ = window.center();
                return Ok(());
            },
            "right" => (monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - 20, monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2),
            "bottom-left" => (monitor_pos.x + 20, monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - 40),
            "bottom-center" => (monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2, monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - 40),
            "bottom-right" => (monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - 20, monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - 40),
            _ => {
                let _ = window.center();
                return Ok(());
            }
        };
        
        window.set_position(tauri::PhysicalPosition { x, y }).map_err(|e| e.to_string())?;
    }
    Ok(())
}
