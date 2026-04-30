import { useState, useEffect } from "react";
import { LayoutGrid, Power, Mic, Maximize, Keyboard, MousePointer2, Sliders, Activity as ActivityIcon, ArrowUpLeft, ArrowUp, ArrowUpRight, ArrowLeft, ArrowRight, ArrowDownLeft, ArrowDown, ArrowDownRight, Moon, Sun, Palette, Globe, Type, Zap, ClipboardCheck, Sparkles } from "lucide-react";
import logo from "./assets/logo.png";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./index.css";

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [windowLabel, setWindowLabel] = useState("");
  const [volume, setVolume] = useState(0);
  const [devices, setDevices] = useState<string[]>([]);
  const [selectedDevice, setSelectedDevice] = useState("Default");
  const [isTesting, setIsTesting] = useState(false);
  const [hotkey, setHotkey] = useState("F8");
  const [isRecordingHotkey, setIsRecordingHotkey] = useState(false);
  const [selectedPosition, setSelectedPosition] = useState("center");
  const [status, setStatus] = useState("Ready");
  
  // Settings
  const [autoType, setAutoType] = useState(true);
  const [activeTab, setActiveTab] = useState<"dashboard" | "settings" | "system">("dashboard");
  const [sensitivity, setSensitivity] = useState(1.0);
  const [darkMode, setDarkMode] = useState(false);
  const [language, setLanguage] = useState("en");
  const [typingSpeed, setTypingSpeed] = useState(10);
  const [autoStart, setAutoStart] = useState(false);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [soundEnabled, setSoundEnabled] = useState(true);
  const [startMinimized, setStartMinimized] = useState(false);
  const [aiMode, setAiMode] = useState(false);

  const applyConfig = (config: any) => {
    setHotkey(config.hotkey);
    setSelectedPosition(config.position);
    if (config.device) setSelectedDevice(config.device);
    setAutoType(config.auto_type);
    setSensitivity(config.sensitivity);
    setDarkMode(config.dark_mode);
    setLanguage(config.language || "en");
    setTypingSpeed(config.typing_speed || 10);
    setAutoStart(config.auto_start || false);
    setMinimizeToTray(config.minimize_to_tray !== undefined ? config.minimize_to_tray : true);
    setSoundEnabled(config.sound_enabled !== undefined ? config.sound_enabled : true);
    setStartMinimized(config.start_minimized || false);
    setAiMode(config.ai_mode || false);
    
    document.documentElement.setAttribute('data-theme', config.dark_mode ? 'dark' : 'light');
    document.documentElement.style.colorScheme = config.dark_mode ? 'dark' : 'light';
  };

  const updateGlobalConfig = async (at: boolean, sens: number, dm: boolean, lang?: string, speed?: number, start?: boolean, mtt?: boolean, se?: boolean, sm?: boolean, ai?: boolean) => {
    const finalLang = lang || language;
    const finalSpeed = speed !== undefined ? speed : typingSpeed;
    const finalStart = start !== undefined ? start : autoStart;
    const finalMtt = mtt !== undefined ? mtt : minimizeToTray;
    const finalSe = se !== undefined ? se : soundEnabled;
    const finalSm = sm !== undefined ? sm : startMinimized;
    const finalAi = ai !== undefined ? ai : aiMode;

    setDarkMode(dm);
    document.documentElement.setAttribute('data-theme', dm ? 'dark' : 'light');

    await invoke("update_config", { 
      autoType: at, 
      sensitivity: sens, 
      darkMode: dm,
      language: finalLang,
      typingSpeed: finalSpeed,
      autoStart: finalStart,
      minimizeToTray: finalMtt,
      soundEnabled: finalSe,
      startMinimized: finalSm,
      aiMode: finalAi
    });
  };

  const [resetFeedback, setResetFeedback] = useState(false);

  const resetToDefaults = async () => {
    const defaultConfig = await invoke("reset_config");
    applyConfig(defaultConfig);
    setResetFeedback(true);
    setTimeout(() => setResetFeedback(false), 2000);
  };

  useEffect(() => {
    const label = getCurrentWindow().label;
    setWindowLabel(label);
    invoke("get_config").then(applyConfig);
    
    const unlistenStatus = listen("recording-status", (event: any) => {
      const payload = event.payload as { recording: boolean; status: string };
      setIsRecording(payload.recording);
      setStatus(payload.status);
      if (!payload.recording) setVolume(0);
    });

    const unlistenVolume = listen("audio-volume", (event: any) => setVolume(event.payload as number));

    const unlistenConfig = listen("config-changed", (event: any) => applyConfig(event.payload));

    const unlistenCopy = listen("copy-to-clipboard", (event: any) => {
      navigator.clipboard.writeText(event.payload as string);
    });

    invoke("get_audio_devices").then((d: any) => setDevices(["Default", ...d]));

    return () => {
      unlistenStatus.then((fn) => fn());
      unlistenVolume.then((fn) => fn());
      unlistenConfig.then((fn) => fn());
      unlistenCopy.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (isRecordingHotkey) {
      const handleKeyDown = (e: KeyboardEvent) => {
        e.preventDefault();
        const modifiers = ["CONTROL", "ALT", "SHIFT", "META", "COMMAND"];
        if (modifiers.includes(e.key.toUpperCase())) return;
        let keys = [];
        if (e.ctrlKey) keys.push("Control");
        if (e.altKey) keys.push("Alt");
        if (e.shiftKey) keys.push("Shift");
        if (e.metaKey) keys.push("Command");
        const mainKey = e.key.toUpperCase().replace(/CONTROL|ALT|SHIFT|META|COMMAND/g, "").trim();
        if (mainKey) keys.push(mainKey);
        if (keys.length > 0) {
          const newHotkey = keys.join("+");
          setHotkey(newHotkey);
          invoke("update_hotkey", { newHotkey });
          setIsRecordingHotkey(false);
        }
      };
      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }
  }, [isRecordingHotkey]);

  const handleDeviceChange = async (name: string) => {
    setSelectedDevice(name);
    await invoke("set_audio_device", { name });
  };

  const handlePositionChange = async (position: string) => {
    setSelectedPosition(position);
    await invoke("set_overlay_position", { position });
  };

  const toggleMicTest = async () => {
    if (isTesting) {
      await invoke("stop_mic_test");
      setIsTesting(false);
      setVolume(0);
      setStatus("Ready");
    } else {
      await invoke("start_mic_test");
      setIsTesting(true);
      setStatus("Testing Mic...");
    }
  };

  if (windowLabel === "main") {
    return (
      <div className="main-dashboard glass" data-theme={darkMode ? 'dark' : 'light'}>
        <aside className="sidebar">
          <div className="logo">
            <div className="logo-icon"><img src={logo} alt="Logo" className="sidebar-logo-img" /></div>
            <span>Swift Speak</span>
          </div>
          <nav>
            <div className={`nav-item ${activeTab === 'dashboard' ? 'active' : ''}`} onClick={() => setActiveTab('dashboard')}>
              <LayoutGrid size={18} /> Dashboard
            </div>
            <div className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`} onClick={() => setActiveTab('settings')}>
              <Sliders size={18} /> Settings
            </div>
            <div className={`nav-item ${activeTab === 'system' ? 'active' : ''}`} onClick={() => setActiveTab('system')}>
              <Zap size={18} className="icon-yellow" /> System
            </div>
          </nav>
          <div className="nav-item quit" onClick={() => invoke('quit_app')}>
            <Power size={18} /> Quit
          </div>
        </aside>
        
        <main className="content">
          <header>
            <div className="header-left">
              <h1>{activeTab === 'dashboard' ? 'Control Center' : activeTab === 'settings' ? 'Settings' : 'System Config'}</h1>
              <p className="subtitle">{activeTab === 'dashboard' ? 'AI Dictation is active and ready' : activeTab === 'settings' ? 'Tailor the engine to your workflow' : 'Manage app behavior and startup'}</p>
            </div>
            <div className={`status-badge ${isRecording || isTesting ? 'active' : ''}`}>
              <div className="pulse-dot" />
              <span>{status}</span>
            </div>
          </header>
          
          <div className="view-container">
            {activeTab === 'dashboard' ? (
              <div className="dashboard-view animate-in">
                <div className="card-row">
                  <div className="settings-card glass-card premium hotkey-hero">
                    <div className="card-header"><Keyboard size={20} className="icon-indigo" /><h3>Activation Key</h3></div>
                    <div className="setting-group">
                      <div className={`hotkey-recorder large ${isRecordingHotkey ? 'recording' : ''}`} onClick={() => setIsRecordingHotkey(true)}>
                        <span className="key-tag">{isRecordingHotkey ? "PRESS KEYS..." : hotkey}</span>
                        <MousePointer2 size={20} />
                      </div>
                    </div>
                  </div>
                  <div className="settings-card glass-card appearance-card">
                    <div className="card-header"><Palette size={20} className="icon-purple" /><h3>Appearance</h3></div>
                    <div className="setting-group">
                      <div className="theme-selection-pill">
                        <button className={`pill-btn ${!darkMode ? 'active' : ''}`} onClick={() => updateGlobalConfig(autoType, sensitivity, false)}><Sun size={18} /><span>Light</span></button>
                        <button className={`pill-btn ${darkMode ? 'active' : ''}`} onClick={() => updateGlobalConfig(autoType, sensitivity, true)}><Moon size={18} /><span>Dark</span></button>
                        <div className="pill-slider" style={{ transform: darkMode ? 'translateX(100%)' : 'translateX(0)' }} />
                      </div>
                    </div>
                  </div>
                </div>
                <div className="card-row">
                  <div className="settings-card glass-card mic-card">
                    <div className="card-header"><div className="title-with-test"><Mic size={20} className="icon-red" /><h3>Audio Engine</h3></div><button onClick={toggleMicTest} className={`test-btn ${isTesting ? 'active' : ''}`}>{isTesting ? "Stop" : "Test Mic"}</button></div>
                    <div className="setting-group">
                      <select value={selectedDevice} onChange={(e) => handleDeviceChange(e.target.value)} className="glass-select">{devices.map(d => <option key={d} value={d}>{d}</option>)}</select>
                      <div className="live-waveform-container">
                        <ActivityIcon size={14} className="activity-icon" /><div className="dashboard-waveform">{[...Array(12)].map((_, i) => (<div key={i} className="wave-bar" style={{ height: `${4 + (volume * 32 * (1 - Math.abs(i - 5.5) / 6))}px`, opacity: 0.3 + (volume * 0.7)}} />))}</div>
                      </div>
                    </div>
                  </div>
                  <div className="settings-card glass-card">
                    <div className="card-header"><LayoutGrid size={20} className="icon-blue" /><h3>Overlay Position</h3></div>
                    <div className="pos-layout">
                      <div className="position-grid-3x3">
                        <button className={selectedPosition === 'top-left' ? 'active' : ''} onClick={() => handlePositionChange('top-left')}><ArrowUpLeft size={18} /></button>
                        <button className={selectedPosition === 'top-center' ? 'active' : ''} onClick={() => handlePositionChange('top-center')}><ArrowUp size={18} /></button>
                        <button className={selectedPosition === 'top-right' ? 'active' : ''} onClick={() => handlePositionChange('top-right')}><ArrowUpRight size={18} /></button>
                        <button className={selectedPosition === 'left' ? 'active' : ''} onClick={() => handlePositionChange('left')}><ArrowLeft size={18} /></button>
                        <button className={selectedPosition === 'center' ? 'active' : ''} onClick={() => handlePositionChange('center')}><Maximize size={18} /></button>
                        <button className={selectedPosition === 'right' ? 'active' : ''} onClick={() => handlePositionChange('right')}><ArrowRight size={18} /></button>
                        <button className={selectedPosition === 'bottom-left' ? 'active' : ''} onClick={() => handlePositionChange('bottom-left')}><ArrowDownLeft size={18} /></button>
                        <button className={selectedPosition === 'bottom-center' ? 'active' : ''} onClick={() => handlePositionChange('bottom-center')}><ArrowDown size={18} /></button>
                        <button className={selectedPosition === 'bottom-right' ? 'active' : ''} onClick={() => handlePositionChange('bottom-right')}><ArrowDownRight size={18} /></button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            ) : activeTab === 'settings' ? (
              <div className="settings-view animate-in">
                <div className="settings-grid">
                  <div className="settings-card glass-card"><div className="card-header"><Globe size={20} className="icon-blue" /><h3>Transcription</h3></div>
                    <div className="setting-group"><span>Recognition Language</span><select value={language} onChange={(e) => { setLanguage(e.target.value); updateGlobalConfig(autoType, sensitivity, darkMode, e.target.value);}} className="custom-select"><option value="en">English (Global)</option><option value="es">Spanish</option><option value="fr">French</option><option value="de">German</option><option value="hi">Hindi</option></select></div>
                    <div className="setting-group"><div className="toggle-item"><span>Automatic Typing</span><div className={`custom-toggle ${autoType ? 'active' : ''}`} onClick={() => { const val = !autoType; setAutoType(val); updateGlobalConfig(val, sensitivity, darkMode); }}><div className="toggle-thumb" /></div></div><p className="setting-hint">If off, results are copied to clipboard.</p></div>
                  </div>
                  <div className="settings-card glass-card">
                    <div className="card-header">
                      <Type size={20} className="icon-purple" />
                      <h3>Typing Behavior</h3>
                    </div>
                    <div className="setting-group">
                      <div className="slider-label"><span>Typing Delay</span><span className="value">{typingSpeed}ms</span></div><input type="range" min="5" max="100" step="5" value={typingSpeed} onChange={(e) => { const val = parseInt(e.target.value); setTypingSpeed(val); updateGlobalConfig(autoType, sensitivity, darkMode, language, val); }} className="premium-slider" /><p className="setting-hint">Recommended: 10ms. Prevents character loss in slow apps.</p>
                    </div>
                    <div className="setting-group">
                      <div className="toggle-item">
                        <div className="title-with-icon">
                          <Sparkles size={16} className="icon-yellow" style={{ marginRight: '8px' }} />
                          <span>AI Quick-Send (Auto-Enter)</span>
                        </div>
                        <div className={`custom-toggle ${aiMode ? 'active' : ''}`} onClick={() => { 
                          const val = !aiMode; setAiMode(val); updateGlobalConfig(autoType, sensitivity, darkMode, language, typingSpeed, autoStart, minimizeToTray, soundEnabled, startMinimized, val); 
                        }}>
                          <div className="toggle-thumb" />
                        </div>
                      </div>
                      <p className="setting-hint">Perfect for ChatGPT/Claude. Hitting enter after typing.</p>
                    </div>
                  </div>
                  <div className="settings-card glass-card"><div className="card-header"><Sliders size={20} className="icon-indigo" /><h3>Engine Sensitivity</h3></div>
                    <div className="setting-group"><div className="slider-label"><span>Microphone Gain</span><span className="value">{sensitivity.toFixed(1)}x</span></div><input type="range" min="0.5" max="3.0" step="0.1" value={sensitivity} onChange={(e) => { const val = parseFloat(e.target.value); setSensitivity(val); updateGlobalConfig(autoType, val, darkMode); }} className="premium-slider" /></div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="system-view animate-in">
                <div className="settings-grid">
                  <div className="settings-card glass-card">
                    <div className="card-header">
                      <Zap size={20} className="icon-yellow" />
                      <h3>System Behavior</h3>
                    </div>
                    <div className="setting-group">
                      <div className="toggle-item">
                        <span>Launch on Startup</span>
                        <div className={`custom-toggle ${autoStart ? 'active' : ''}`} onClick={() => {
                          const val = !autoStart; setAutoStart(val); updateGlobalConfig(autoType, sensitivity, darkMode, language, typingSpeed, val);
                        }}><div className="toggle-thumb" /></div>
                      </div>
                    </div>
                    <div className="setting-group">
                      <div className="toggle-item">
                        <span>Minimize to Tray on Close</span>
                        <div className={`custom-toggle ${minimizeToTray ? 'active' : ''}`} onClick={() => {
                          const val = !minimizeToTray; setMinimizeToTray(val); updateGlobalConfig(autoType, sensitivity, darkMode, language, typingSpeed, autoStart, val);
                        }}><div className="toggle-thumb" /></div>
                      </div>
                    </div>
                    <div className="setting-group">
                      <div className="toggle-item">
                        <span>Start Minimized</span>
                        <div className={`custom-toggle ${startMinimized ? 'active' : ''}`} onClick={() => {
                          const val = !startMinimized; setStartMinimized(val); updateGlobalConfig(autoType, sensitivity, darkMode, language, typingSpeed, autoStart, minimizeToTray, soundEnabled, val);
                        }}><div className="toggle-thumb" /></div>
                      </div>
                    </div>
                    <div className="setting-group">
                      <p className="setting-hint">App will start directly in the task tray with optional chimes.</p>
                    </div>
                  </div>

                  <div className="settings-card glass-card danger-card">
                    <div className="card-header"><Power size={20} className="icon-red" /><h3>Maintenance</h3></div>
                    <button 
                      className={`reset-btn-large ${resetFeedback ? 'success' : ''}`} 
                      onClick={resetToDefaults}
                    >
                      {resetFeedback ? "Reset Successful!" : "Reset to Factory Defaults"}
                    </button>
                    <p className="setting-hint">This will clear all your custom hotkeys and preferences.</p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </main>
      </div>
    );
  }

  const isActuallyTyping = status === "Typing..." || status === "Copied!";
  const isIntermediate = status === "Transcribing..." || status === "Processing...";
  const isOverlayVisible = isRecording || isActuallyTyping || isIntermediate;

  return (
    <div key={`overlay-${darkMode ? 'dark' : 'light'}-${isRecording ? 'active' : 'idle'}`} className={`app-container ${isOverlayVisible ? "visible" : "hidden-state"} ${(isRecording || isIntermediate) ? "recording" : ""} ${isActuallyTyping ? "typing" : ""}`} data-theme={darkMode ? 'dark' : 'light'} data-tauri-drag-region style={{ "--voice-volume": volume } as any}>
      {isActuallyTyping ? (
        <div className="typing-view animate-in">
          {status === "Copied!" ? <ClipboardCheck size={18} className="typing-icon icon-green" /> : <Keyboard size={18} className="typing-icon" />}
          <span className="typing-text">{status}</span>
        </div>
      ) : isIntermediate ? (
        <div className="loader-view animate-in"><div className="glow-spinner" /></div>
      ) : (
        <div className="waveform animate-in">{[...Array(8)].map((_, i) => (<div key={i} className="bar" />))}</div>
      )}
    </div>
  );
}

export default App;
