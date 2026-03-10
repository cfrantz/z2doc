# Docassembler: NES Disassembly Analysis Tool

## Introduction
**Docassembler** is a high-performance, web-based disassembly analysis tool specifically designed for 8-bit Nintendo Entertainment System (NES) games. Built with **Rust** and the **Leptos** framework, it runs entirely within your browser as a WebAssembly (WASM) application. It requires no backend server and offers a desktop-like experience for reverse-engineering.

## Getting Started
Docassembler operates on local files using the Browser File System Access API.

1.  **Open Database:** On the setup screen, select a `.json` or `.json5` database file. If configured by your deployment, you may also see "Remote Database" buttons for quick loading.
2.  **Open ROM:** Provide the `.nes` ROM file associated with the project.
3.  **Persistence:** If you opened a local database, clicking **Save** will write changes directly to that file. If you loaded a remote database, **Save** will trigger a "Save As" dialog to create a local copy.

## Interface Overview
The interface is designed for high-density information display with a persistent control header.

### Header Controls
- **Bank Selector:** Switch between PRG banks.
- **Search Bar:** Real-time search across symbols, operands, and comments.
- **Theme Selector:** Toggle between Light and Dark modes.
- **Help Button:** Opens project documentation (README) in a new tab.
- **Save Button:** Persists all annotations to your database file.

### Disassembly Columns
- **Addr:** PRG Bank and CPU Address (e.g., `$02:$8354`).
- **Bytes:** Raw hexadecimal instruction or data bytes.
- **Op:** Instruction mnemonic or pseudo-op (e.g., `.byt`, `.word`).
- **Operand:** Instruction operand, resolved to a symbol name if available.
- **Comment:** Line-specific documentation.

## Search and Navigation

### Search Functionality
The search bar provides real-time filtering of the current bank.
- **ENTER:** Find the next occurrence.
- **CTRL + ENTER:** Find the previous occurrence.
- **Visual Feedback:** All matches are highlighted. The currently active match is highlighted in a brighter color.

### Cross-References (Symbol Links)
Operand values that resolve to known symbols are hyperlinked.
- **Click** a link to navigate to that symbol's definition. The tool automatically handles bank switching and scrolling.
- **Deep Linking:** The URL hash (`#bank-XX-addr-XXXX`) updates automatically as you scroll or navigate, supporting browser history and bookmarks.

## Annotation & Editing

### In-Place Editing
Most fields in the disassembly grid are interactive.
- **Click** any symbol or comment to begin editing.
- **ENTER** or **Blur (Click Away)**: Commit the change to the database.
- **ESCAPE**: Discard the current edit and revert to the previous text.

### Advanced Operations (Shift + Click)
- **Rename Symbol:** **Shift + Click** a symbol in the **Operand** column to rename the target symbol globally.
- **Block Comments:** **Shift + Click** an **Address, Hex, or Comment** cell to add or edit a multi-line block comment above that line.

## Customization

### Themes
Docassembler includes reactive **Light** and **Dark** themes. The theme affects all UI elements, including search highlights and link colors. Your preference is saved to local storage.

### Column Resizing
The width of every column can be customized to suit your screen.
- **Drag** the vertical handles in the grid header to resize.
- Column widths are persisted uniquely for each project database.

## Project Organization

### Global vs. Banked Symbols
- **Global Symbols:** Used for RAM variables (`$0000-$07FF`), PPU/APU registers, and mapper registers. These are visible and searchable from any bank.
- **Banked Symbols:** Used for ROM code and data specific to a single PRG bank.

### Auto-labels
Instructions jumping to unnamed addresses are automatically labeled (e.g., `L815A`). These can be renamed at any time to provide descriptive context to the code flow.
