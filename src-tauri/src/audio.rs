use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;

pub fn record_audio(app: tauri::AppHandle, buffer: Arc<Mutex<Vec<f32>>>, is_recording: Arc<Mutex<bool>>, sample_rate_arc: Arc<Mutex<u32>>, channels_arc: Arc<Mutex<u16>>, device_name: Option<String>) {
    let host = cpal::default_host();
    
    let device = if let Some(name) = device_name {
        host.input_devices().expect("failed to list devices")
            .find(|d| d.name().unwrap_or_default() == name)
            .unwrap_or_else(|| host.default_input_device().expect("no input device available"))
    } else {
        host.default_input_device().expect("no input device available")
    };
    
    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to get default input config: {}", e);
            return;
        }
    };

    let sr = config.sample_rate().0;
    let ch = config.channels();
    println!("Recording starting: {}Hz, {} channels using device: {}", sr, ch, device.name().unwrap_or_default());
    
    *sample_rate_arc.lock().unwrap() = sr;
    *channels_arc.lock().unwrap() = ch;
    
    let is_recording_clone = is_recording.clone();
    let buffer_clone = buffer.clone();
    let app_handle = app.clone();

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            if *is_recording_clone.lock().unwrap() {
                // Calculate volume (RMS)
                let mut sum = 0.0;
                if !data.is_empty() {
                    for &sample in data {
                        sum += sample * sample;
                    }
                    let rms = (sum / data.len() as f32).sqrt();
                    let normalized_volume = (rms * 25.0).min(1.0); // Higher sensitivity for better bounce
                    use tauri::Emitter;
                    let _ = app_handle.emit("audio-volume", normalized_volume);
                }

                let mut buf = buffer_clone.lock().unwrap();
                for frame in data.chunks(ch as usize) {
                    buf.push(frame[0]);
                }
            }
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        None,
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to play stream");
    
    while *is_recording.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Recording thread stopped.");
}

pub fn save_to_wav(data: Vec<f32>, input_sample_rate: u32, app_data_dir: PathBuf) -> Result<PathBuf, String> {
    let target_sample_rate = 16000;
    let spec = WavSpec {
        channels: 1,
        sample_rate: target_sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let path = app_data_dir.join("input.wav");
    let mut writer = WavWriter::create(&path, spec).map_err(|e| e.to_string())?;

    // Find max amplitude for normalization
    let mut max_val = 0.0f32;
    for &sample in &data {
        max_val = max_val.max(sample.abs());
    }
    println!("Swift Speak: Max audio amplitude: {:.4}", max_val);
    
    // Boost normalization: Aim for 0.9 amplitude even with quiet input
    let multiplier = if max_val > 0.001 { 0.9 / max_val } else { 1.0 };

    // Resample to 16kHz
    let ratio = input_sample_rate as f32 / target_sample_rate as f32;
    let mut index = 0.0;
    
    while (index as usize) < data.len() {
        let sample = data[index as usize];
        let normalized = (sample * multiplier).clamp(-1.0, 1.0);
        let amplitude = (normalized * i16::MAX as f32) as i16;
        writer.write_sample(amplitude).map_err(|e| e.to_string())?;
        index += ratio;
    }
    
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(path)
}
