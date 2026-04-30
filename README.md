# Swift Speak 🎙️

Swift Speak is a minimalist, high-performance local dictation application built with Tauri and the Whisper model. It allows you to capture audio via a global hotkey, transcribe it locally using your GPU, and automatically type the text into any active application.

## Features

- **Local & Private**: Your audio never leaves your machine. Transcription is done locally using OpenAI's Whisper model.
- **Global Hotkey**: Start dictating from anywhere with a simple shortcut.
- **High Performance**: Optimized for NVIDIA GPUs (RTX 4060+) for near-instant transcription.
- **Seamless Integration**: Automatically types transcribed text into your active window.
- **Modern UI**: A sleek, dark-themed dashboard and floating overlay.

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri
- **AI Engine**: Local Whisper (via Candle/Faster-Whisper)
- **Styling**: Vanilla CSS

## Installation

1. Download the latest release from the [Releases](https://github.com/USER_NAME/REPO_NAME/releases) page.
2. Run the installer (`.msi` or `.exe`).
3. Follow the setup instructions to configure your global hotkey.

## Development

To run the project locally:

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
