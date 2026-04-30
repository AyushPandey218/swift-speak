use std::path::Path;

pub async fn transcribe_local(wav_path: &Path, resource_dir: &Path, app_data_dir: &Path, language: &str) -> Result<String, String> {
    let sidecar_path = resource_dir.join("whisper-main.exe");
    let model_path = resource_dir.join("ggml-base.bin");
    
    if !sidecar_path.exists() {
        return Err(format!("Whisper engine missing in resources: {:?}", sidecar_path));
    }

    // -np (no prints) is critical to only get the transcribed text
    println!("Swift Speak: Running engine for language: {}", language);
    
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new(&sidecar_path);
    
    // Add resource_dir to PATH so the .exe can find its .dll files
    let mut path_var = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = std::env::split_paths(&path_var).collect::<Vec<_>>();
    paths.push(resource_dir.to_path_buf());
    let new_path = std::env::join_paths(paths).unwrap();
    
    cmd.env("PATH", new_path)
        .current_dir(app_data_dir) // Use writable AppData for execution
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
