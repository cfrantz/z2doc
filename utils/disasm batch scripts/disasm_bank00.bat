del zelda_ii_bank00.asm
type nes_registers.info zelda_ii_common.info zelda_ii_cartridge_ram.info zelda_ii_bank00.info zelda_ii_bank07.info > temp.info
da65 -i temp.info -o zelda_ii_bank00.asm -S $8000 zelda_ii_bank00_rom.bin
pause
del temp.info
