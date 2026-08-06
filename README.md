# Alfred

A local, voice-enabled personal AI assistant.

Alfred combines a local LLM (Ollama), local speech-to-text (OpenAI's Whisper), text-to-speech (Coqui XTTS v2), and a long-term memory system to provide a conversational assistant experience—via keyboard or voice—without mandatory cloud dependency.

## Why this project?

This project grew out of a personal challenge: to build my own AI assistant while knowing exactly where my data is hosted and how it is used. Running the LLM, speech recognition, and text-to-speech locally directly addresses this goal eliminating reliance on third-party services for core processing.

This project was developed in my spare time, primarily for my own use. It is neither designed nor maintained with the aim of broad community distribution; the code, technical choices, and documentation reflect my specific needs and constraints rather than a goal of general-purpose utility. The project will evolve over time based on my availability and interests, without a fixed roadmap or guarantees of stability. 

## Features

- **Local LLM** via [Ollama](https://ollama.com) (default model: `qwen3:4b-instruct`)
- **Local speech recognition** via [whisper-rs](https://github.com/tazz4843/whisper-rs) (whisper.cpp)
- **Text-to-speech** via [Coqui XTTS v2](https://github.com/coqui-ai/TTS), exposed as a local HTTP microservice (Python script not provided)
- **Long-term memory**: periodic consolidation of interactions ("Dream") into a structured summary, re-injected at startup
- **Tool registry** (tool calling) to connect Alfred to external services
- **Functional CLI** (text and voice)

## Architecture

The project is a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) composed of several independent crates sharing a common core.

- **`alfred-core`**: shared library: interaction orchestration, long-term memory, tool registry, speech transcription, microphone capture. Does not depend on any specific interface.
- **`alfred-cli`**: command-line interface (text and/or voice).
- **`alfred-web`** *(coming soon)* — HTTP/WebSocket API for a web interface.

## Prerequisites

| Tool | Usage |
|---|---|
| [Rust](https://www.rust-lang.org/) (2021 edition) | Project compilation |
| [Ollama](https://ollama.com) | Local LLM execution |
| [CMake](https://cmake.org/) | whisper.cpp compilation |
| [LLVM/Clang](https://releases.llvm.org/) | Binding generation via `bindgen` |
| Python 3.11+ with a virtual environment | TTS microservice (Coqui XTTS v2) |
| [FFmpeg](https://ffmpeg.org/) *(optional)* | Conversion of test audio files |

> **Windows**: Install CMake and LLVM via `winget`, then **fully restart the terminal** to refresh the `PATH`.
> ```powershell
> winget install Kitware.CMake
> winget install LLVM.LLVM
> ```

## Installation

### 1. Clone the repository

```bash
git clone <repo-url>
cd alfred-assistant
```

### 2. Download the LLM model

```bash
ollama pull qwen3:4b-instruct
```

### 3. Download the Whisper model

```powershell
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin" -OutFile "stt-models/ggml-small.bin"
```

### 4. Set up the TTS microservice

```powershell
cd tts-server
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install fastapi uvicorn TTS
```

### 5. Environment variables

Create a `.env` file in the root directory:
Regardez le fichier .env.example

### 6. Build the workspace

```bash
cargo build
```

> ⚠️ The initial build compiles whisper.cpp (C++) and may take several minutes. ## Usage

### Start dependent services

```powershell
# Terminal 1: Ollama
ollama serve

# Terminal 2: TTS microservice
cd tts-server
.\venv\Scripts\python.exe server_tts.py
```

### Launch Alfred (CLI)

```bash
cargo run --bin alfred-cli
```

## Project structure

```
alfred-assistant/
├── Cargo.toml              # Workspace manifest
├── alfred-core/             # Shared library
│   └── src/
│       ├── core/             # Alfred, Orchestrator, DreamTimer
│       ├── services/         # memory, ollama, stt, micro, application
│       └── commands/         # Tool registry and handlers
├── alfred-cli/               # Command-line interface
│   └── src/
├── tts-server/                # TTS microservice (Python)
├── stt-models/                 # Whisper models (.bin)
├── skills/                     # Alfred system prompt
└── datas/                      # SQLite database (history, memory)
```

## Roadmap

Some ideas I have in mind for adding features:

- [x] LLM orchestration with tool calling
- [x] Long-term memory ("Dream" consolidation)
- [x] Local speech recognition (whisper-rs)
- [x] Speech synthesis (microservice)
- [x] Split into Cargo workspace (`alfred-core` / `alfred-cli`)
- [ ] Wake word detection ("Alfred, ...")
- [ ] Web API (`alfred-web`) and web interface
- [ ] Integrations: Gmail, Google Calendar, Spotify
- [ ] Home automation
- [ ] Proactive behavior

## License

This project is distributed under a custom non-commercial license.

You are permitted to use, modify, and share this project solely for non-commercial purposes, subject to the following conditions:

Obtain prior authorization from the author before any use.
Clearly credit me in your project or documentation.
Notify me when you use this project.
Any commercial use is prohibited without written authorization.

Consult the LICENSE file for the full terms and conditions.