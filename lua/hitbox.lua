-- Simple hitbox script for FCEUX by CF207
-- Based on Fon2d2's analysis of the hitbox code
-- https://discordapp.com/channels/426982216808267776/427134920327168001/476584345520308225
--
local running = true;
local tick = emu.framecount();
local sizetable = 0xe8fa;
local swordbox = {r=255,g=0,b=0,a=64};
local shieldbox = {r=0,g=255,b=0,a=64};
local hitbox = {r=0,g=0,b=255,a=64};

while (running) do
    local ypos = {};
    local xpos = {};
    local xscr = {};
    local exists = {};
    local entype = {};
    local enid = {};
    local scroll;
    local linkstanding = 0;
    local linkfacing = 0; -- 0=right, 1=left
    local gamestate;
    local x, y, w, h;
    local swordx, swordy;
    local controller;

    gamestate = memory.readbyte(0x736);
    if (gamestate == 0x0b) then
        -- gamestate 0x0b is sideview areas.
        controller = memory.readbyte(0x80);
        scroll = memory.readword(0x72c, 0x72a);
        exists[0] = 1;
        linkstanding = memory.readbyte(0x17)
        linkfacing = memory.readbyte(0x9f) - 1;
        swordx = memory.readbyte(0x47e);
        swordy = memory.readbyte(0x480);
        for i=0,13 do
            ypos[i] = memory.readbyte(0x29 + i);
            xpos[i] = memory.readword(0x4d + i, 0x3b + i);
            xscr[i] = xpos[i] - scroll;
            if xscr[i] < 0 or xscr[i] > 255 then
                xscr[i] = -1;
            end
            if i > 0 and i < 7 then
                exists[i] = memory.readbyte(0xb6 + i - 1);
                -- enemy id at ($a1+i), enemy size type at $6e1d+id
                enid[i] = memory.readbyte(0xa1 + i - 1);
                entype[i] = memory.readbyte(0x6e1d + enid[i]);
            elseif i >= 7 then
                exists[i] = memory.readbyte(0x87 + i - 7);
                -- Projectiles use the high bits for part of the collision routine
                if exists[i] > 15 then
                    exists[i] = 0;
                end
                enid[i] = memory.readbyte(0x87 + i - 7);
                entype[i] = memory.readbyte(0x6e1d + enid[i]);
            end
        end

        -- Scroll offset and link position.
        -- gui.text(8, 8, string.format("SC=%03d Lx=%03d Ly=%03d", scroll, xpos[0], ypos[0]));
        -- Link's hitbox
        x = 9; w = 13;
        y = memory.readbyte(0xe971 + linkstanding);
        h = memory.readbyte(0xe973 + linkstanding);
        gui.rect(xscr[0]+x, ypos[0]+y, xscr[0]+x+w, ypos[0]+y+h, hitbox);

        -- Compute shield defense box
        if linkfacing == 0 then
            x = 8+14
        else
            x = 8-1
        end
        if linkstanding == 1 then
            y = 2
        else
            y = 17
        end
        w = 5;
        h = 12;
        gui.rect(xscr[0]+x, ypos[0]+y, xscr[0]+x+w, ypos[0]+y+h, shieldbox);

        -- Compute sword hit box
        if swordy ~= 0xF8 then
            w = 14; h = 3;
            if swordx > xscr[0] then
                x = -8;
            else
                x = 2;
            end
            if controller == 0x09 then
                y = 0;
            else
                y = 7;
            end
            gui.rect(swordx+x, swordy+y, swordx+x+w, swordy+y+h, swordbox);
        end

        -- The rest of the hitboxes
        for i=1,13 do
            if xscr[i] ~= -1 and exists[i] ~= 0 then
                local ofs = AND(entype[i] * 4, 255);
                x = memory.readbytesigned(sizetable + ofs + 0);
                w = memory.readbytesigned(sizetable + ofs + 1);
                y = memory.readbytesigned(sizetable + ofs + 2);
                h = memory.readbytesigned(sizetable + ofs + 3);
                gui.rect(xscr[i]+x, ypos[i]+y, xscr[i]+x+w, ypos[i]+y+h, hitbox);
                -- Enemy ID and spawn index
                -- gui.text(xscr[i], ypos[i]-8, string.format("e=%d i=%d", enid[i], i));
            end
        end
    end
    FCEU.frameadvance();
end

