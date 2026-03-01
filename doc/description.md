# NES Disassembly Analysis Application

# Introduction

This program will be a disassembly analysis program for 8-bit Nintendo Entertainment System (NES) games.  The program will consist of two main parts: a backend web server (written in rust) and a frontend user interface using web technologies (html, css, javascript, and the Alpine.js framework).  The program will allow the user to view the disassembly of a NES game and edit the disassembly metadata (symbol names, comments, etc).  User edits will be sent to the backend system and stored in a database of disassembly metadata.  The backend system will maintain the database of metadata, serialized as JSON5 text files on the filesystem to support hexadecimal values and comments. The backend will use that metadata to update the disassembly displayed to the user.

# Backend

The backend server will be written in rust and use a popular rust web-backend framework such as [Rocket](https://rocket.rs/).  The backend will be responsible for maintaining a database of disassembly metadata (including receiving metadata updates from the frontend), reading a NES ROM and creating a disassembly with the metadata and serving up both the user interface and the disassembly. The path to the NES ROM should be provided as a command-line argument when starting the backend server.

The NES is an 8-bit machine with a 6502 CPU (technically the CPU is a Ricoh 2A03, but the distinction is not relevant to this program).  It is typical for games to use a memory mapper chip such as [MMC1](https://www.nesdev.org/wiki/MMC1) or [MMC3](https://www.nesdev.org/wiki/MMC3) to allow the CPU to access more than 64K of address space.  On the NES, game code is in ROM and the ROM is mapped into the CPU address space from 0x8000 to 0xFFFF (inclusive).  Typically the mapper chips subdivide this 32K region into 16K or 8K chunks.  It is typical for the high bank (starting at 0xC000 or 0xE000 depending on the subdivision scheme) to have a fixed mapping and for the low banks (starting at 0x8000, 0xA000 and sometimes 0xC000) to be switched during game execution.  The backend and database must be aware of this bank switching scheme in order to properly categorize symbols that appear in switchable banks.  For uniformity’s sake, we’ll consider the high bank to be switchable even though that almost never happens.

The NES has RAM located at cpu addresses 0x0000 to 0x07FF (inclusive).  NES game cartridges sometimes supplied additional ram located at 0x6000 to 0x7FFF (inclusive).  The NES also has peripherals in the CPU address space: the video chip (PPU), the audio chip (APU) and the IO ports that are responsible for reading game controller inputs.  Neither the RAM nor peripherals are subject to the bank switching schemes of the mapper.  As such, the database that associates comments or symbol names with addresses should not consider that these addresses have an associated bank number.

NES ROMs are typically stored in what is commonly called the “NES File” or “iNES File” format.  This format has a 16-byte “[NES 2.0](https://www.nesdev.org/wiki/NES_2.0)” header followed by a number of program (or PRG) banks followed by a number of graphic character (or CHR) banks.  Regardless of the banking capabilities of the mapper specified in the NES 2.0 header, the header conceptualizes the PRG and CHR bank sizes as 16K and 8K respectively.  It is rare for classic NES game ROMs to use any of the features encoded in bytes 8 to 16 of the NES 2.0 header.  Handling the features encoded in these bytes is out-of-scope for this program.

# Frontend

The frontend user interface allows the user to view and annotate the disassembly.  The user interface should present a view of the disassembly as a program listing (e.g. a sequence of lines) with different components divided into columns.  These columns include:

- A bank number and cpu address displayed in hexadecimal and separated by a colon.  As is the convention with the 6502 cpu, the hex values should be preceded by a dollar-sign: `$02:$8354`.  
- The raw hex bytes that make up the disassembled instruction, separated by spaces.  In this case, the dollar-sign convention should be ignored  
- The opcode of the instruction.  
- The operand of the instruction.  If the operand corresponds to a known symbol, the symbol name should be displayed.  If the operand does not correspond to a known symbol, then the operand should be displayed in hexadecimal with the leading dollar-sign.  
- A comment corresponding to that line of code (or more precisely, corresponding to the code at that specific bank/cpu address).

There are some exceptions to this columnar display scheme:

- A symbol name that corresponds to a code address should be displayed left-justified should appear to be on the line prior to the code display.  
- A (possibly multiline) block comment associated with an address should also be displayed left-justified and appear prior to the code associated with that address.  In the case where both a symbol name and a block comment exist, the block comment should be displayed first.

Operands that refer to a symbol name should be hyperlinked to the definition of that symbol. If the symbol is located in a different bank than the one currently displayed, clicking the link should automatically navigate to the target bank.
The frontend should only display a single bank at a time. The user should be able to navigate between banks using a dropdown selector. The full disassembly may be large enough to overwhelm the web browser client.

The annotating tasks allow the user to edit the comment fields, add new comments or apply names to symbols.  I imagine that these edits will be done by using the `contentEditable` attribute of the underlying HTML elements.  The frontend should synchronize edits to the backend using an "auto-save on blur" approach, which will update the symbol & comment database.  When symbols are edited, the frontend should refresh its display to update reference to a new or renamed symbol.

## Display Theme

Each displayable element should be themeable by the user.  There should be two built-in themes: light-mode and dark-mode.

In light-mode:

* The background should be white  
* The address and raw hex bytes text should be grey  
* The instruction and opcode text should be black  
* The comment text should be grey  
* Symbol names should be blue.  The blue color should be applied both when symbols are displayed alone on a single line and when a symbol is displayed as part of an operand.

In dark-mode:

* The background should be a dark grey (maybe \#202020).  
* The address and raw hex bytes should be light grey (maybe \#909090).  
* The instruction and opcode text should be white  
* The comment text should be light grey.  
* Symbol names should be blue.  The blue color should be applied both when symbols are displayed alone on a single line and when a symbol is displayed as part of an operand.

The backend should maintain display theme data structures and use them to build an appropriate theming cascading style sheet that is served dynamically to the front end.  The theme data structures should be serializable and saved in a separate user configuration file (JSON5) on disk, independent of the disassembly metadata database. This allows different users to maintain their own themes. A theme editor in the front-end is currently out-of-scope.

# Database

In this document, the term “database” is being used generically and doesn’t necessarily imply a specific database technology like SQL.  In fact, I prefer that the database be maintained in a set of JSON5 text files.

I expect that the data structures held by the database to be plain rust data structures and that the rust `serde` crate, along with a JSON5 library, will be used to serialize and deserialize the database between the text representation and the in-memory runtime representation.
In order to facilitate disassembly, the database should be able to represent symbols and comments that are associated with cpu addresses.  Some of those cpu addresses (namely those in the ROM region) are subject to the bank switching scheme of the mapper, and thus should also be categorized by bank.  The data associated with each cpu address is any combination of a symbol name, a comment or a block comment. Non-banked addresses (RAM, PPU, APU, and other peripherals) are stored in a "global" section. An initial template database file should be provided, pre-populated with definitions for the standard NES registers ($2000-$4017).

The database should also encode whether or not a given address region contains code, data as bytes or data as words.  The program should assume any address region not defined contains data as bytes.

One possible way to represent this data is as follows:  
```
enum RegionInfo {  
    Code(std::ops::RangeInclusive<u16>),  
    Bytes(std::ops::RangeInclusive<u16>),  
    Words(std::ops::RangeInclusive<u16>),  
}

struct AnnotationInfo {  
    pub symbol: Option<String>,  
    pub comment: Option<String>,  
    pub block_comment: Option<String>,  
}

type SectionInfo = BTreeMap<u16, AnnotationInfo>

struct BankInfo {  
    // List of code and data regions to aid disassembly.  
    pub region: Vec<RegionInfo>,  
    // Symbol and comment information for ROM code and data.  
    pub address: SectionInfo,  
}

struct DisassemblyInfo {  
    // Symbol and comment information for RAM variables and global peripherals
    pub global: SectionInfo,  
    // Information for ROM code and data. The u8 key is the bank index 
    // based on the configured mapper window size (8K or 16K).
    pub bank: BTreeMap<u8, BankInfo>,  
}  
```
# Disassembly

The backend should provide a disassembly service for the frontend, creating disassembly data in a form convenient to transform into the display representation.  As mentioned elsewhere in this document, a single disassembly of an entire NES ROM will likely overwhelm the web browser, so the disassembly service should provide a limited subset of the entire disassembly. Limiting the data by bank is a reasonable strategy.

The disassembler should only disassemble regions explicitly marked as `Code` in the `RegionInfo` for the relevant bank. Any region not explicitly defined should be treated as `Bytes` by default. The initial disassembly should incorporate the standard NES registers defined in the `global` section of the template database.

