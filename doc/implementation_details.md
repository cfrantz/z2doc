# Implementation Details

## 1. Project Structure
The project will be a combined Rust/Web application using the following structure:
```text
docusemmbler/
├── Cargo.toml
├── src/
│   ├── main.rs         // Rocket entry point and API routes
│   ├── database.rs     // JSON5 persistence and DB management
│   ├── disassembler.rs // 6502 disassembly logic
│   └── models.rs       // Shared data structures (AnnotationInfo, etc.)
├── static/
│   ├── index.html      // Alpine.js UI
│   ├── style.css       // Base styles and dynamic theme overrides
│   └── app.js          // Client-side Alpine.js logic
└── templates/
    └── default_db.json5 // Initial pre-populated database template
```

## 2. Disassembly Representation
The backend will serve a JSON array of instruction objects to the frontend, optimized for Alpine.js rendering:
```json
[
  {
    "address": "$02:$8354",
    "bytes": "4C 00 80",
    "opcode": "JMP",
    "operand": "$8000",
    "symbol": "Reset",
    "comment": "Main entry point",
    "block_comment": null
  }
]
```

## 3. Annotation Synchronization
- **Trigger:** Frontend uses the `blur` event on `contentEditable` elements.
- **Action:** Alpine.js sends a `POST` request to `/api/annotation` with the updated metadata.
- **Backend:** Updates the in-memory `DisassemblyInfo`, persists to the JSON5 file, and returns a success status.
- **Refresh:** Frontend re-fetches the current bank's disassembly to reflect any renamed symbols or cascading changes.

## 4. Mapper Configuration
- **Storage:** The mapper window size (8K or 16K) will be stored in the database JSON5 file itself.
- **Default:** If missing, a default of 16K will be assumed.

## 5. Core Data Structures
We'll use `serde` and `json5-rs` for serialization.
- **`DisassemblyInfo`**: The root database struct.
  - `global`: `SectionInfo` (for RAM, PPU, APU, and non-banked peripherals).
  - `bank`: `BTreeMap<u8, BankInfo>` (indexed by bank).
  - `mapper_window_size`: `u8` (8 or 16).
- **`AnnotationInfo`**: 
  - `symbol`: `Option<String>`
  - `comment`: `Option<String>`
  - `block_comment`: `Option<String>`
- **`RegionInfo`**: Enum with variants:
  - `Code(RangeInclusive<u16>)`
  - `Bytes(RangeInclusive<u16>)`
  - `Words(RangeInclusive<u16>)`
- **`ThemeConfig`**: A separate struct for the user theme configuration, defining colors for background, text, and interactive elements.

## 6. Disassembly Service
The service will transform raw ROM bytes and metadata into a display-ready format:
- **Code Regions**: Linear sweep disassembly. Instruction operands are resolved to symbols by checking the local bank's `address` map first, then the `global` map.
- **Data Regions (Bytes)**:
  - Grouped into pseudo-opcodes (e.g., `.byt $01, $02...`) with up to 8 bytes per line.
  - **Symbol Boundary Rule**: If a symbol is defined at an address within a sequence, the sequence MUST be broken to ensure the symbol label appears at the start of a new line.
- **Data Regions (Words)**:
  - Formatted as 16-bit little-endian words (e.g., `.word $1234`).
  - Like bytes, sequences are broken if a symbol is defined at a specific address.
- **Address Resolution**:
  - Operands that point to a valid address within a `Code` region are hyperlinked.
  - If the target address is in a different bank, the link will include metadata to trigger a bank switch in the frontend.

## 7. API Endpoints
- **`GET /api/disassembly/<bank_id>`**: Returns the JSON array of instruction/data objects for the requested bank.
- **`POST /api/annotation`**: Updates an `AnnotationInfo` in the database.
  - Payload: `{ bank_index: Option<u8>, address: u16, data: AnnotationInfo }`.
- **`GET /api/theme.css`**: Returns a dynamically generated CSS file based on the current `ThemeConfig`.
- **`GET /api/metadata`**: Returns general project information (ROM filename, total banks, mapper window size) for UI initialization.
