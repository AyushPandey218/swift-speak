use std::path::Path;

pub async fn transcribe_local(wav_path: &Path, app_data_dir: &Path, language: &str) -> Result<String, String> {
    let mut sidecar_path = app_data_dir.join("whisper-main.exe");
    let mut model_path = app_data_dir.join("ggml-base.bin");
    
    // Smart detection: If not in current folder, check the legacy folder
    if !sidecar_path.exists() {
        let legacy_dir = app_data_dir.parent().unwrap().join("com.ayush.tauri-app");
        if legacy_dir.exists() {
            let legacy_sidecar = legacy_dir.join("whisper-main.exe");
            let legacy_model = legacy_dir.join("ggml-base.bin");
            if legacy_sidecar.exists() && legacy_model.exists() {
                println!("Swift Speak: Found AI engine in legacy folder. Using that.");
                sidecar_path = legacy_sidecar;
                model_path = legacy_model;
            }
        }
    }
    
    if !sidecar_path.exists() {
        return Err("Whisper engine (whisper-main.exe) missing. Please move it to the new AppData folder or use the 'Download' button in settings.".to_string());
    }

    // -np (no prints) is critical to only get the transcribed text
    println!("Swift Speak: Running engine for language: {}", language);
    
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new(&sidecar_path);
    cmd.current_dir(app_data_dir)
        .arg("-m")
        .arg(&model_path)
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
