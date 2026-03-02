# Project Docusemmbler: NES Disassembly Analysis Tool

## 1. Project Structure
The project is a combined Rust/Web application with a modular backend and an Alpine.js-powered frontend.
```text
docusemmbler/
├── Cargo.toml
├── src/
│   ├── main.rs         // Rocket entry point and API routes
│   ├── database.rs     // JSON5 persistence and DB management
│   ├── models.rs       // Shared data structures (AnnotationInfo, etc.)
│   └── disasm/
│       └── mod.rs      // 6502 disassembly logic and opcode tables
├── static/
│   ├── index.html      // Alpine.js UI
│   ├── style.css       // Base styles and CSS variables for themes
│   └── app.js          // Client-side Alpine.js logic
└── templates/
    └── default_db.json // Initial pre-populated database template
```

## 2. Disassembly Representation
The backend serves a JSON array of `DisassemblyLine` objects, optimized for granular rendering:
```json
[
  {
    "address_label": "$02:$8354",
    "address": 33620,
    "bank": 2,
    "bytes": "4C 00 80",
    "opcode": "JMP",
    "operand_prefix": "",
    "operand_main": "Reset",
    "operand_suffix": "",
    "operand_is_symbol": true,
    "symbol": "Reset",
    "comment": "Main entry point",
    "block_comment": null,
    "target_bank": 2,
    "target_address": 32768
  }
]
```

## 3. Annotation & Navigation
- **In-Place Editing:** Frontend uses Alpine.js `contentEditable` fields for symbols and comments.
- **Synchronization:** On `blur`, a `POST` request is sent to `/api/annotation`. The backend persists changes and returns a signal to refresh the disassembly.
- **Hash-Based Navigation:** The UI uses URL hashes (`#bank-XX-addr-XXXX`) to permit deep-linking, back/forward navigation, and synchronized scrolling.
- **Symbol Links:** Operands resolved to symbols are hyperlinked. Clicking a link triggers a bank switch (if necessary) and scrolls the target address into view.

## 4. Mapper Configuration
- **Window Size:** Supports 8K or 16K PRG banking, stored in the database.
- **Fixed Banks:** One or more banks can be marked as `is_fixed` (e.g., the $C000-$FFFF range in many NES mappers).
- **Fixed Range:** The `mapper_fixed_range` field defines the CPU address space occupied by fixed banks.

## 5. Core Data Structures
- **`DisassemblyInfo`**: Root database struct.
  - `global`: `SectionInfo` (Annotations for RAM, PPU, etc.).
  - `bank`: `BTreeMap<u8, BankInfo>` (Indexed by PRG bank).
  - `mapper_window_size`: `u8` (8 or 16).
  - `mapper_fixed_range`: `Option<RangeInclusive<u16>>`.
- **`BankInfo`**:
  - `is_fixed`: `bool` (Is this bank always mapped?).
  - `mapped_at`: `Option<u16>` (Default CPU mapping address).
  - `region`: `Vec<RegionInfo>` (List of Code/Data ranges).
  - `address`: `SectionInfo` (Local annotations).
- **`RegionInfo`**: Enum: `Code(Range)`, `Bytes(Range)`, `Words(Range)`.

## 6. Disassembly Service
The `disasm` module performs sophisticated linear sweep disassembly:
- **Gap Filling:** Any address range not explicitly covered by a `RegionInfo` is automatically treated as a `Bytes` region.
- **Auto-Labeling:** Discovered branch/jump targets without user-defined symbols are automatically labeled as `LXXXX`.
- **Symbol Resolution Priority:**
    1. Local bank annotations.
    2. Fixed bank annotations (if the target address is in the fixed range).
    3. Global annotations.
- **Sequence Breaking:** `Bytes` and `Words` sequences are broken if a symbol or auto-label is defined at an internal address, ensuring labels always start a new line.

## 7. API Endpoints
- **`GET /api/metadata`**: Returns project title, ROM filename, bank list, and mapper configuration.
- **`GET /api/disassembly/<bank_id>`**: Returns the disassembly for a bank (or `255` for global symbols).
- **`POST /api/annotation`**: Updates `AnnotationInfo`. Payload: `{ bank_id, address, symbol, comment, block_comment }`.
- **`GET /api/themes`**: Returns a list of available theme names.
- **`POST /api/themes/active`**: Sets the active theme for the session.
- **`GET /api/theme.css`**: Dynamically generates CSS based on the active `ThemeConfig` (colors for opcodes, symbols, comments, etc.).
