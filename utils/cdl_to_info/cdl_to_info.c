/*---------------------------------------------------------------------
 |  cdl_to_info.c
 |
 |  Author:  fon2d2
 |
 |  Purpose:  Take a .cdl file output by the FCEUX code/data logger
 |      and convert to a .info file readable by the da65 disassembler.
 |
 |  Requirements before running cdl_to_info:  You must allow the code/
 |      data logger to capture as much info as possible.  Load the ROM
 |      you wish to disassemble, open the Code/Data Logger from the
 |      Debug menu and begin logging.  You can use the auto save/load
 |      feature so that you do not need to complete the game in one
 |      setting.
 |    Once you what you consider to be a complete CDL file, you must
 |      manually break this apart into separate CDL files for each ROM
 |      bank.  I use HxD for this.
 |
 |  How to use:  Invoke from the command line as follows:
 |      ./cdl_to_info <cdl file> > <info file>
 |
 |  Assumptions made by cdl_to_info:
 |      - ROM banks are 0x2000 bytes and start at either 0x8000 or
 |        0xC000.  (E.g. as with Zelda II)
 |      - Proper address assignments can be inferred from the address
 |        map bits and only the address map bits of the first byte
 |        need to be checked.
 |      - Any byte not flagged as code will be added in a BYTETABLE
 |        RANGE statement.
 |      - Any byte labelled as both code and data will be added in
 |        a CODE RANGE statement.
 |
 |  Output:  .info file covering the entire range of the ROM bank.
 |      The output file contains a consecutive series of RANGE
 |      statements alternating between CODE and BYTETABLE.
 |      
 |  Recommendations for use:  Create additional .info files for RAM
 |      spaces and NES PPU/APU registers.  Add LABEL statements for
 |      RAM addresses, byte tables in ROM, and subroutines as they
 |      become known.  Rerun the disassembler as LABEL statements
 |      are added.  When disassembling, specify all relevant .info
 |      files.
 |    For example, to disassemble a Zelda II ROM bank, I would want
 |      to specify .info files for main and cartridge RAM, NES
 |      registers, and ROM bank 7, since ROM bank 7 is always present.
 |
 |  External Documentation:
 |    FCEUX Code/Data Logger:
 |      http://www.fceux.com/web/help/fceux.html?CodeDataLogger.html
 |    da65 Info File Format:
 |      https://cc65.github.io/doc/da65.html#ss3.4
 |
 *-------------------------------------------------------------------*/


#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

#define ADDRESS_MASK        0x0C
#define DATA_MASK           0x02
#define CODE_MASK           0x01

#define ADDRESS_SHIFT       2
#define ADDRESS_BLOCK_SIZE  0x2000
#define ADDRESS_BASE        0x8000

void usage(char * progName);

int main(int argc, char *argv[])
{
    FILE *      cdlFile;
    uint8_t     byteRead;
    uint8_t     addressFlags;
    bool        isCodeBlock;
    bool        isCodeByte;
    uint16_t    startAddressOfThisROMBank;
    uint16_t    startAddressOfThisBlock;
    uint16_t    currentAddress;
    
    if(argc < 2) {
        usage(argv[0]);
        exit(1);
    }
    
    cdlFile = fopen(argv[1], "rb");
    if(NULL == cdlFile) {
        printf("Failed to open %s\r\n", argv[1]);
        exit(1);
    }
    
    fread(&byteRead, sizeof(uint8_t), 1, cdlFile);
    
    addressFlags = byteRead & ADDRESS_MASK;
    startAddressOfThisROMBank = (addressFlags >> ADDRESS_SHIFT) * ADDRESS_BLOCK_SIZE +
        ADDRESS_BASE;
    
    startAddressOfThisBlock = startAddressOfThisROMBank;
    currentAddress = startAddressOfThisROMBank;
    
    isCodeBlock = (byteRead & CODE_MASK) ? true : false;
    
    currentAddress++;
    
    while(!feof(cdlFile) && (currentAddress < 0x10000)) {
        fread(&byteRead, sizeof(uint8_t), 1, cdlFile);
        isCodeByte = (byteRead & CODE_MASK) ? true : false;
        if(isCodeByte != isCodeBlock) {
            // Changed from code to data or vica-versa
            if(isCodeBlock) {
                printf("RANGE { START $%4X;    END   $%4X; TYPE Code;      };\r\n",
                    startAddressOfThisBlock, currentAddress - 1);
            } else {
                printf("RANGE { START $%4X;    END   $%4X; TYPE ByteTable; };\r\n",
                    startAddressOfThisBlock, currentAddress - 1);
            }
            isCodeBlock = isCodeByte;
            startAddressOfThisBlock = currentAddress;
        }
        currentAddress++;
    }
    
    fclose(cdlFile);
    
    return 0;
}

void usage(char * progName)
{
    printf("%s <inputfile>\r\n", progName);
    printf("Redirect stdout to desired da65 compatible info file\r\n");
}
