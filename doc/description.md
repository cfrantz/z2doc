# NES Disassembly Analysis Application

## Introduction

This program is a disassembly analysis tool for 8-bit Nintendo Entertainment System (NES) games. It is implemented as a high-performance WebAssembly (WASM) application using the **Leptos** framework. 

The application is entirely client-side, meaning it requires no specialized backend server and can be hosted as a set of static files (e.g., via GitHub Pages or a simple local web server). It allows users to view the disassembly of an NES game, edit metadata (symbol names, comments, etc.), and persist those changes directly to their local filesystem using modern browser APIs.

## Architecture

The application is built using a unified Rust codebase compiled to WebAssembly.

- **Frontend Framework:** [Leptos](https://leptos.dev/), a full-stack, isomorphic Rust web framework that provides fine-grained reactivity and a component-based architecture.
- **Build Tool:** [Trunk](https://trunkrs.dev/), which manages WASM compilation, asset bundling, and the development lifecycle.
- **Reactivity:** Uses Leptos "Signals" to manage global state (ROM data, database annotations, current bank, etc.) and ensure the UI stays synchronized with data changes.
- **Portability:** Because all disassembly and logic occur in WASM, the tool can be hosted anywhere that serves static files.

## File Access & Persistence

The application bridges the gap between web and desktop apps by utilizing the **Browser File System Access API**.

### Local Operation
When the application starts without pre-loaded data, it guides the user through a setup flow:
1. **Open Database:** The user selects a `.json` (or JSON5) metadata file.
2. **Open ROM:** The user selects the corresponding `.nes` ROM file.
Once handles are established, the application can save changes directly back to the local database file without "download" prompts, providing a seamless desktop-like experience.

### Remote Loading
The application supports loading a database from a URL via a query parameter (e.g., `?db=https://example.com/zelda2.json`). 
- In this mode, the application fetches the remote JSON and then prompts the user for the local ROM file.
- Changes made to a remote database can be saved to the user's local disk via a "Save As" flow (using `showSaveFilePicker`).

## User Interface

The UI presents a structured view of the disassembly with several key features:

### Columnar Disassembly View
The disassembly is displayed in a program listing with components divided into columns:
- **Address:** The bank number and CPU address in hex (e.g., `$02:$8354`).
- **Bytes:** The raw hex bytes of the instruction.
- **Opcode:** The 6502 instruction mnemonic.
- **Operand:** The instruction operand, resolved to a symbol name if known.
- **Comment:** A line-specific comment.

### Specialized Rows
- **Symbols:** Code-level symbol names appear left-justified on the line preceding the code.
- **Block Comments:** Multi-line comments appear left-justified before the associated symbol or code.
- **Global Equates:** Non-banked symbols (e.g., RAM or hardware registers) are displayed as equations (e.g., `PPU_CTRL = $2000`).

### Performance: Virtualized Scrolling
To handle large PRG banks (which can exceed 8,000 lines), the UI employs a **high-performance virtual scrolling** engine.
- Only a small window of rows (plus a buffer) is actually rendered in the DOM at any given time.
- Row heights are pre-calculated to account for variable-sized elements (symbols and block comments).
- This ensures a smooth, 60fps scrolling experience regardless of the total bank size.

### Navigation
- **Hyperlinks:** Operands referring to symbols are hyperlinked. Clicking a link triggers a bank switch and scrolls the target address into view.
- **Hash-based Routing:** The application uses URL hashes (`#bank-XX-addr-XXXX`) to support deep-linking and browser back/forward history.

## Display Themes

The UI is fully themeable via CSS variables.
- **Built-in Themes:** Includes standard Light and Dark modes.
- **Persistence:** The active theme preference is stored locally.
- **Dynamic CSS:** Theme configurations (colors for opcodes, symbols, comments, etc.) are applied reactively in WASM, updating the UI instantly without page reloads.

## Database & Disassembly Logic

The core logic for parsing iNES headers, 6502 disassembly, and metadata management is implemented in pure Rust.

### Data Model
Metadata is stored in a `DisassemblyInfo` structure, which includes:
- **Global Section:** Annotations for non-banked regions (RAM, PPU, etc.).
- **Banked Section:** Metadata for ROM regions, indexed by PRG bank.
- **Region Info:** Definitions for whether an address range contains `Code`, `Bytes`, or `Words`.

### Disassembly Service
The disassembler performs a linear sweep of the ROM:
- **Gap Filling:** Any range not explicitly marked as `Code`, `Bytes`, or `Words` defaults to `Bytes`.
- **Auto-Labeling:** Branch/jump targets without user-defined symbols are automatically labeled (e.g., `L8354`).
- **Symbol Resolution:** Symbols are prioritized by local bank, then fixed banks, then global equates.
