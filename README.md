# Swift Speak 🎙️

[![Rust](https://img.shields.io/badge/rust-%23E32F26.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=tauri&logoColor=FFFFFF)](https://tauri.app/)
[![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)](https://reactjs.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

**Swift Speak** is a minimalist, high-performance local dictation application built for speed and privacy. By leveraging OpenAI's Whisper model locally, it provides a seamless "voice-to-text" experience without ever sending your audio to the cloud.

---

## ✨ Features

- **🔒 100% Private**: All transcription is done locally on your device. Your audio never leaves your machine.
- **⚡ Blazing Fast**: Optimized for modern NVIDIA GPUs (RTX 4060+) for near-instant transcription.
- **⌨️ Global Hotkey**: Start dictating instantly from any application with a custom shortcut (default: `F8`).
- **🎯 Auto-Typing**: Automatically types transcribed text into your active window. Perfect for ChatGPT, Emails, or Coding.
- **🌈 Modern UI**: Sleek, glassmorphic dashboard with Dark/Light mode support.
- **📦 Zero Setup**: Comes bundled with the AI model and engine—just install and speak.

---

## 🚀 Installation

1. Go to the [Releases](https://github.com/AyushPandey218/swift-speak/releases) page.
2. Download the latest `Swift Speak_x64-setup.exe`.
3. Run the installer and launch the app.
4. Set your preferred hotkey and start dictating!

---

## 🛠️ Technical Architecture

Swift Speak uses a multi-layered architecture for maximum performance:
- **Frontend**: React + TypeScript + Vite for a responsive, modern interface.
- **Backend**: Rust (Tauri) for low-level system access and performance.
- **AI Engine**: A bundled `whisper.cpp` main executable acting as a local inference server.
- **Communication**: Tauri's secure command system handles the bridge between the UI and the AI engine.

---

## 💻 Development

### Prerequisites
- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://www.rust-lang.org/tools/install)
- [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (for Windows)

### Setup
```bash
# Clone the repository
git clone https://github.com/AyushPandey218/swift-speak.git

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build
```bash
npm run tauri build
```

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙌 Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request if you have ideas for new features or optimizations.

*Made with ❤️ by [Ayush](https://github.com/AyushPandey218)*
