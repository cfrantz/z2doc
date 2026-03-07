# Project Docassembler: NES Disassembly Analysis Tool

## 1. Project Structure
The project is a pure client-side Rust WebAssembly (WASM) application built with the **Leptos** framework and bundled using **Trunk**.
```text
docassembler/
├── Cargo.toml          // WASM dependencies and Trunk config
├── index.html          // Application entry point and Trunk asset link
├── src/
│   ├── main.rs         // Leptos UI components, state management, and setup flow
│   ├── database.rs     // JSON parsing and serialization (WASM compatible)
│   ├── models.rs       // Shared data structures (AnnotationInfo, DisassemblyLine, etc.)
│   └── disasm/
│       └── mod.rs      // 6502 disassembly logic and opcode tables
├── static/
│   └── style.css       // Base styles and CSS variables for themes and layout
└── templates/
    └── default_db.json // Initial pre-populated database template
```

## 2. Architecture & Persistence
- **Client-Side execution:** All disassembly, logic, and rendering occur entirely within the browser via WASM. No backend server is required.
- **File Access API:** Uses the **Browser File System Access API** to permit direct read/write access to local `.json` database files and `.nes` ROM files.
- **State Management:** Uses Leptos **Signals** and **Memos** for fine-grained reactivity. The `AppState` struct holds global state (ROM data, database, current bank, etc.).
- **Persistence:** 
    - **Database:** Saved directly to the local filesystem via the file handle.
    - **Settings:** Column widths and theme preferences are persisted in `localStorage`.

## 3. Disassembly & Virtual Scrolling
To handle large NES PRG banks (8K or 16K) which can contain thousands of lines, the application uses a **custom virtualized scrolling engine**:
- **Variable Height Support:** Row heights are pre-calculated to account for block comments and symbols.
- **Dynamic Rendering:** Only a small "window" of rows (plus a buffer) is rendered in the DOM at any time, maintaining 60fps performance.
- **Binary Search Offsets:** Uses binary search on pre-calculated Y-offsets to quickly determine which instructions are visible for a given scroll position.

## 4. Annotation & Navigation
- **In-Place Editing:** Uses HTML `contentEditable` fields managed by Leptos. Changes to symbols or comments reactively update the underlying database signal.
- **Programmatic Scrolling:** Clicking a symbol link (operand) triggers a navigation signal. The virtualized list calculates the target offset and programmatically scrolls the container.
- **Cross-Bank Links:** Navigation supports switching banks or jumping to the "Global Symbols" pseudo-bank automatically.
- **Auto-Labeling:** Discovered branch/jump targets without user-defined symbols are automatically labeled as `LXXXX`.

## 5. Mapper & Data Model
- **`DisassemblyInfo`**: Root database struct.
  - `global`: `BTreeMap<u16, AnnotationInfo>` (Non-banked symbols like RAM/PPU).
  - `bank`: `BTreeMap<u8, BankInfo>` (Indexed by PRG bank).
  - `mapper_window_size`: `u8` (8 or 16).
  - `mapper_fixed_range`: `Option<RangeInclusive<u16>>`.
- **`RegionInfo`**: Enum defining memory ranges: `Code`, `Bytes`, or `Words`.
- **Gap Filling:** Any address range not explicitly covered by a `RegionInfo` is automatically disassembled as a `Bytes` region.

## 6. UI & Themes
- **Column Resizing:** Interactive resizing of columns (Addr, Bytes, Op, Operand) via mouse drag, implemented using CSS variables and global mouse listeners.
- **Reactive Theming:** Dynamic CSS injection allows switching between built-in themes (Light/Dark) or user themes instantly.
- **Layout:** High-performance flexbox and grid layout ensures the disassembly fills the available viewport while the header remains fixed.
