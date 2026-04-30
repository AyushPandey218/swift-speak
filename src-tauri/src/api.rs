use std::path::Path;

pub async fn transcribe_local(wav_path: &Path, resource_dir: &Path, app_data_dir: &Path, language: &str) -> Result<String, String> {
    let engine_in_appdata = app_data_dir.join("whisper-main.exe");
    let model_in_appdata = app_data_dir.join("ggml-base.bin");

    // If files are missing in AppData, copy them from the bundled Resources
    if !engine_in_appdata.exists() || !model_in_appdata.exists() {
        println!("Swift Speak: Initializing engine files in AppData...");
        let _ = std::fs::create_dir_all(app_data_dir);
        
        // Copy everything from resources to AppData
        if let Ok(entries) = std::fs::read_dir(resource_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let dest = app_data_dir.join(path.file_name().unwrap());
                    let _ = std::fs::copy(&path, &dest);
                }
            }
        }
    }

    if !engine_in_appdata.exists() {
        return Err(format!("Engine initialization failed. Could not find whisper-main.exe"));
    }

    // -np (no prints) is critical to only get the transcribed text
    println!("Swift Speak: Running engine for language: {}", language);
    
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    // Now run from AppData (Guaranteed writable and DLLs are in the same folder)
    let mut cmd = std::process::Command::new(&engine_in_appdata);
    
    cmd.current_dir(app_data_dir) // Use writable AppData for execution
        .arg("-m")
        .arg(&model_in_appdata)
        .arg("-f")
        .arg(wav_path.to_str().unwrap())
        .arg("-l")
        .arg(language)
        .arg("-np") 
        .arg("-nt");

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let output = cmd.output()
        .map_err(|e| {
            println!("Swift Speak: CRITICAL - Failed to start engine process: {}", e);
            e.to_string()
        })?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("Swift Speak: Engine Success. Transcribed text: '{}'", text);
        if text.is_empty() {
             println!("Swift Speak: Engine returned empty text.");
             return Err("No text detected. Try speaking louder!".to_string());
        }
        Ok(text)
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        println!("Swift Speak: Engine FAILED with status: {:?}", output.status);
        println!("Swift Speak: Engine Error Output: {}", err);
        Err(format!("Engine error: {}", err))
    }
}
