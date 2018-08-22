del zelda_ii_bank04.asm
type nes_registers.info zelda_ii_common.info zelda_ii_cartridge_ram.info zelda_ii_bank04.info zelda_ii_bank07.info > temp.info
da65 -i temp.info -o zelda_ii_bank04.asm -S $8000 zelda_ii_bank04_rom.bin
pause
del temp.info
