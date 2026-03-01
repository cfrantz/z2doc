document.addEventListener('alpine:init', () => {
    Alpine.data('editField', () => ({
        editing: false,
        startEdit() {
            this.editing = true;
            this.$nextTick(() => {
                this.$el.focus();
                // Place cursor at end
                const range = document.createRange();
                const sel = window.getSelection();
                range.selectNodeContents(this.$el);
                range.collapse(false);
                sel.removeAllRanges();
                sel.addRange(range);
            });
        },
        stopEdit(line, field) {
            this.editing = false;
            this.$dispatch('update-annotation', { line, field, value: this.$el.innerText });
        }
    }));

    Alpine.data('disasmApp', () => ({
        metadata: { name: '', title: '', rom_file: '', total_banks: 0, mapper_window_size: 0 },
        currentBank: 0,
        disassembly: [],
        themes: [],
        currentTheme: 'Dark',

        // Resizing state
        resizing: null,
        startX: 0,
        startWidth: 0,
        colWidths: {
            addr: 100,
            hex: 200,
            op: 60,
            operand: 150
        },

        async init() {
            await this.fetchMetadata();
            await this.fetchThemes();
            
            // Load saved column widths
            const saved = localStorage.getItem(this.metadata.name + '.colWidths');
            if (saved) {
                this.colWidths = JSON.parse(saved);
            }

            await this.fetchDisassembly();
        },

        async fetchThemes() {
            const response = await fetch('/api/themes');
            this.themes = await response.json();
            if (this.themes.includes('User')) {
                this.currentTheme = 'User';
            }
        },

        async setTheme() {
            await fetch('/api/themes/active', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: this.currentTheme })
            });
            // Force CSS reload by appending timestamp
            const link = document.getElementById('theme-link');
            link.href = '/api/theme.css?t=' + new Date().getTime();
        },

        async fetchMetadata() {
            const response = await fetch('/api/metadata');
            this.metadata = await response.json();
        },

        async fetchDisassembly() {
            const response = await fetch(`/api/disassembly/${this.currentBank}`);
            this.disassembly = await response.json();
        },

        async changeBank() {
            await this.fetchDisassembly();
            document.getElementById('disasm-container').scrollTop = 0;
        },

        async updateAnnotation(line, field, value) {
            const processedValue = this.stripDecorations(field, value);

            // Only update if value actually changed to avoid redundant saves
            if (line[field] === processedValue || (line[field] === null && processedValue === "")) return;

            const req = {
                bank_id: line.bank === -1 ? null : parseInt(this.currentBank),
                address: line.address,
                symbol: field === 'symbol' ? processedValue : line.symbol,
                comment: field === 'comment' ? processedValue : line.comment,
                block_comment: field === 'block_comment' ? processedValue : line.block_comment
            };

            // Convert empty strings to null for the backend
            if (req.symbol === "") req.symbol = null;
            if (req.comment === "") req.comment = null;
            if (req.block_comment === "") req.block_comment = null;

            const response = await fetch('/api/annotation', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(req)
            });

            if (response.ok) {
                await this.fetchDisassembly();
            }
        },

        navigate(targetBank, targetAddress) {
            const scroll = (addr) => {
                const symElement = document.getElementById(`sym-${addr}`);
                if (symElement) {
                    symElement.scrollIntoView();
                } else {
                    const addrElement = document.getElementById(`addr-${addr}`);
                    if (addrElement) addrElement.scrollIntoView();
                }
            };

            if (targetBank !== null && targetBank !== this.currentBank) {
                this.currentBank = targetBank;
                this.fetchDisassembly().then(() => {
                    this.$nextTick(() => scroll(targetAddress));
                });
            } else {
                scroll(targetAddress);
            }
        },

        startResizing(col, e) {
            this.resizing = col;
            this.startX = e.pageX;
            this.startWidth = this.colWidths[col];
        },

        resize(e) {
            if (!this.resizing) return;
            const diff = e.pageX - this.startX;
            this.colWidths[this.resizing] = Math.max(20, this.startWidth + diff);
        },

        stopResizing() {
            if (this.resizing) {
                localStorage.setItem(this.metadata.name + '.colWidths', JSON.stringify(this.colWidths));
            }
            this.resizing = null;
        },

        get gridStyles() {
            return {
                '--col-addr': `${this.colWidths.addr}px`,
                '--col-hex': `${this.colWidths.hex}px`,
                '--col-op': `${this.colWidths.op}px`,
                '--col-operand': `${this.colWidths.operand}px`
            };
        },

        stripDecorations(field, text) {
            if (!text) return text;
            if (field === 'symbol') {
                let v = text.trim();
                if (v.endsWith(':')) {
                    v = v.slice(0, -1);
                }
                return v;
            }
            if (field === 'comment' || field === 'block_comment') {
                return text.split('\n').map(line => {
                    let l = line.trimStart();
                    if (l.startsWith(';')) {
                        l = l.substring(1);
                        if (l.startsWith(' ')) {
                            l = l.substring(1);
                        }
                    }
                    return l;
                }).join('\n').trimEnd();
            }
            return text;
        }
    }));
});
