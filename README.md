# Docassembler: NES Disassembly Analysis Tool

## Introduction
**Docassembler** is a modern, web-based disassembly analysis tool specifically designed for 8-bit Nintendo Entertainment System (NES) games. It combines a high-performance Rust backend with an interactive, Alpine.js-powered frontend to provide a seamless reverse-engineering experience.

### AI-assisted coding
This is the author's first foray into AI-assisted coding.
My initial prompt is in the [description](doc/description.md) document.
I didn't write any of the code for this application - I delegated everything to Gemini.  For topics where my own knowledge is somewhat lacking (such as javascript UI frameworks), I even solicited recommendations from Gemini about which frameworks to use and why.

## Getting Started
To use the tool, you must provide a NES ROM file as a command-line argument when starting the backend server:

```bash
./docassembler path/to/game.nes
```

The tool will automatically create or load a project database (a `.json` file) to store your symbols and comments.

## Interface Overview
The user interface is divided into a fixed header and a scrollable disassembly grid.

### Columns
- **Addr:** Displays the PRG Bank and CPU Address (e.g., `$02:$8354`).
- **Bytes:** The raw hexadecimal bytes of the instruction or data.
- **Op:** The instruction mnemonic (e.g., `LDA`, `JSR`) or pseudo-op (e.g., `.byt`, `.word`).
- **Operand:** The instruction operand, resolved to a symbol name if available.
- **Comment:** A line-specific description of the code or data.

## Navigation

### Bank Selection
Use the dropdown menu in the header to switch between different PRG banks. Selecting a new bank will automatically scroll the view to the top of that bank.

### Cross-References (Symbol Links)
Operand values that resolve to known symbols are hyperlinked. 
- **Click** a link to navigate to that symbol's definition.
- If the symbol is in a different bank, the tool will automatically switch banks and scroll to the target address.

### Deep Linking & History
The URL in your browser's address bar updates automatically as you scroll or navigate. You can copy this URL (including the `#bank-XX-addr-XXXX` hash) to bookmark a specific location or share it with others. The browser's **Back** and **Forward** buttons work as expected for navigating your history.

## Annotation & Editing

### Editing Symbols and Comments
Most text in the disassembly is editable.
- **Click** on a symbol label, a comment cell, or a block comment to begin editing.
- **Auto-save:** Changes are sent to the backend as soon as you "blur" (click away from) the field.
- **Pending State:** While an update is being processed by the server, the text will appear with a **strikethrough**. Once the server confirms the change, the UI will refresh and the strikethrough will disappear.
- **Cancel:** Press the **ESCAPE** key while editing to discard your changes and revert to the previous text.

### Symbol Renaming (Operand Field)
You can rename a symbol directly from its reference in the operand field:
1. **Right-click** a symbol name in the **Operand** column.
2. Select **Rename Symbol**.
3. Type the new name and press **Enter**. All references to this symbol across the entire disassembly will be updated.

### Block Comments
Block comments provide high-level documentation and appear above the code they describe.
1. **Right-click** an **Address** cell or a **Symbol** label.
2. Select **Add Block Comment**.
3. A new, full-width field will appear. Type your comment and click away to save.
4. For multi-line comments, use **Ctrl+Enter** to commit the change, or simply click away.

## Customization

### Themes
Switch between **Light**, **Dark**, and **User** themes using the selector in the header. Themes change the color scheme for addresses, opcodes, symbols, and comments.

### Column Resizing
You can customize the width of the columns to suit your screen size.
- **Drag** the vertical handles in the column headers to resize.
- Your preferred column widths are saved automatically and will persist across sessions.

## Project Organization

### Global vs. Banked Symbols
- **Global Symbols:** Used for RAM variables (`$0000-$07FF`), PPU/APU registers, and mapper registers. These are available in all banks.
- **Banked Symbols:** Used for ROM code and data. These are specific to a single PRG bank.

### Auto-labels
If an instruction jumps to an address that hasn't been named yet, the tool automatically generates a temporary label in the format `LXXXX` (e.g., `L815A`). You can rename these at any time to provide a more descriptive label.
