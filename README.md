# Mnemo

> Your clipboard, with memory.

Mnemo is a **local-first, privacy-respecting clipboard assistant** that turns scattered snippets, links, terminal logs, and notes into an intelligent research memory. Rather than just holding the last text you copied, Mnemo remembers the context, reconstructs your research sessions, relates information over time, and answers questions using only your saved memories.

---

## 🌟 Key Features

- **Privacy-First & Opt-In Capture**: By default, clipboard capture is paused. Enable it manually when you begin research, and pause it when you're done. Your data never leaves your device.
- **Semantic & Keyword Search**: Press `Ctrl+Shift+V` (or use the search bar) to search through your clipboard history by exact words or semantic meaning.
- **Research Sessions & Timeline**: Mnemo reconstructs your workflow by grouping clips into continuous, logical timeline sessions, showing how your research progressed.
- **Interactive Memory Graph**: Explore semantic and temporal connections visually. See how a snippet copied today relates to a concept from last week.
- **Grounded AI Answers (Offline-first / Hybrid)**: Ask Mnemo questions. It generates answers grounded strictly in your captured evidence, citing the exact source clips.
  - **Local/Offline AI**: Uses a local semantic model (downloaded automatically post-onboarding) or Ollama.
  - **Cloud Optional**: Connect OpenAI or Gemini if desired; cloud access is strictly opt-in and sends only relevant snippets for that query.
- **Browser Context Bridge**: Attach web page titles and verified URLs to your copies. Install the optional extension (supporting Chrome, Brave, Edge, and Firefox) to bridge browser context via a loopback-only (`127.0.0.1:17531`) connection.

---

## 🚀 Getting Started

### Installation (Private Beta)

1. Download the signed Linux AppImage from the latest GitHub Release.
2. Make it executable and launch it:
   ```bash
   chmod +x Mnemo.AppImage
   ./Mnemo.AppImage
   ```

### Quick Walkthrough

1. **Complete Onboarding**: Start the app. The local semantic model starts downloading in the background. Keyword search and regular features are ready immediately!
2. **Enable Capture**: Toggle **Clipboard Capture** from the sidebar or press `Ctrl+Shift+M`.
3. **Save Memories**: Go ahead and copy code snippets, documentation paragraphs, or terminal output.
4. **Browse Timeline**: Open the **Timeline** tab to see your clipboard history organized neatly into sessions.
5. **Ask/Search**: Press `Ctrl+Shift+V` to open Search. Ask a question like *"How do I configure the AI provider?"* and watch Mnemo answer using your copied clips as citations.
6. **Optional Browser Bridge**: Follow the setup in [`extension/README.md`](extension/README.md) to enable page title and URL metadata integration.

---

## 🛡️ Privacy & Security Commitments

- **No Cloud Required**: Mnemo functions completely offline. Your clipboard history, database, and embedding index live solely on your machine.
- **Manual Toggle & Hotkeys**:
  - `Ctrl+Shift+M`: Quickly toggle clipboard capture on/off.
  - `Ctrl+Shift+V`: Instantly search your memories.
- **Explicit Cloud Consent**: If you configure a cloud AI provider (like OpenAI or Gemini), Mnemo will request permission before each query and will only send relevant evidence snippets.
- **Automatic Retention**: Configure database auto-cleanup in Settings (defaults to retaining history indefinitely).

---

## 🛠️ Development Setup

To run and build Mnemo from source:

### Prerequisites
- Node.js (v18+)
- Rust stable
- Tauri platform prerequisites

### Run Locally
```bash
# Install dependencies
npm install

# Start the application in development mode
npm run tauri dev
```

### Run Tests & Build
```bash
# Run unit and integration tests
npm test

# Build production distribution
npm run build

# Package the browser extension
npm run package:extension -- --verify

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 📄 License

Mnemo is distributed under the MIT License. See [LICENSE](LICENSE) for details.
