document.addEventListener('alpine:init', () => {
    Alpine.data('editField', () => ({
        editing: false,
        
        startEdit() {
            this.editing = true;
            this.$nextTick(() => {
                const editable = this.$el.getAttribute('contenteditable') === 'true' 
                    ? this.$el 
                    : this.$el.querySelector('[contenteditable="true"]');
                
                if (editable) {
                    editable.focus();
                    const range = document.createRange();
                    const sel = window.getSelection();
                    range.selectNodeContents(editable);
                    sel.removeAllRanges();
                    sel.addRange(range);
                }
            });
        },
        stopEdit(line, field) {
            this.editing = false;
            const newValue = this.$el.innerText;
            const processedValue = this.stripDecorations(field, newValue);
            
            // Update local state immediately if possible
            if (line[field] !== undefined) {
                line[field] = processedValue;
            }
            
            this.$dispatch('update-annotation', { line, field, value: newValue });
        },
        stripDecorations(field, text) {
            if (!text) return text;
            if (field === 'symbol') {
                let v = text.trim();
                if (v.endsWith(':')) v = v.slice(0, -1);
                return v;
            }
            if (field === 'comment' || field === 'block_comment') {
                return text.split('\n').map(line => {
                    let l = line.trimStart();
                    if (l.startsWith(';')) {
                        l = l.substring(1);
                        if (l.startsWith(' ')) l = l.substring(1);
                    }
                    return l;
                }).join('\n').trimEnd();
            }
            return text;
        }
    }));

    Alpine.data('disasmApp', () => ({
        metadata: { name: '', title: '', rom_file: '', total_banks: 0, mapper_window_size: 0, banks: {} },
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

        // Context Menu state
        contextMenu: {
            show: false,
            x: 0,
            y: 0,
            target: null
        },

        async init() {
            await this.fetchMetadata();
            await this.fetchThemes();
            
            // Load saved column widths
            const saved = localStorage.getItem(this.metadata.name + '.colWidths');
            if (saved) {
                this.colWidths = JSON.parse(saved);
            }

            // Handle initial hash or fetch default
            if (window.location.hash) {
                await this.handleHash();
            } else {
                await this.fetchDisassembly();
            }

            // Listen for back/forward and manual hash changes
            window.addEventListener('hashchange', () => this.handleHash());
        },

        async handleHash() {
            const hash = window.location.hash;
            if (!hash) return;

            const bankMatch = hash.match(/bank-([0-9A-Fa-f]{2})/);
            const addrMatch = hash.match(/addr-([0-9A-Fa-f]{4})/);

            if (bankMatch) {
                const bankId = bankMatch[1] === 'FF' ? 255 : parseInt(bankMatch[1], 16);
                if (bankId !== parseInt(this.currentBank)) {
                    this.currentBank = bankId;
                    await this.fetchDisassembly();
                }
            }

            if (addrMatch) {
                const addr = parseInt(addrMatch[1], 16);
                this.$nextTick(() => {
                    this.scrolltoAddress(addr);
                });
            }
        },

        scrolltoAddress(addr) {
            const symElement = document.getElementById(`sym-${addr}`);
            if (symElement) {
                symElement.scrollIntoView();
            } else {
                const addrElement = document.getElementById(`addr-${addr}`);
                if (addrElement) addrElement.scrollIntoView();
            }
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
            const bankHex = parseInt(this.currentBank).toString(16).toUpperCase().padStart(2, '0');
            window.location.hash = `bank-${bankHex}`;
        },

        async updateAnnotation(line, field, value) {
            // stripDecorations is shared via editField, but we need it here too
            const processedValue = this.stripDecorations(field, value);

            // Determine correct bank_id
            let bank_id = null;
            if (line.bank === -1) {
                bank_id = null; // Global
            } else if (line.bank !== undefined && line.bank !== null) {
                bank_id = line.bank;
            } else {
                bank_id = parseInt(this.currentBank);
                if (bank_id === 255) bank_id = null;
            }

            const req = {
                bank_id: bank_id,
                address: line.address,
                [field]: processedValue === null ? "" : processedValue
            };

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
            const bankId = targetBank === null || targetBank === -1 ? 255 : targetBank;
            const bankHex = bankId.toString(16).toUpperCase().padStart(2, '0');
            const addrHex = targetAddress.toString(16).toUpperCase().padStart(4, '0');
            window.location.hash = `bank-${bankHex}-addr-${addrHex}`;
        },

        onScroll() {
            if (this.resizing) return;
            
            const container = document.getElementById('disasm-container');
            if (!container) return;
            const cells = container.querySelectorAll('.grid-cell.address');
            let topAddr = null;
            
            const containerRect = container.getBoundingClientRect();
            for (let cell of cells) {
                const rect = cell.getBoundingClientRect();
                if (rect.top >= containerRect.top) {
                    topAddr = cell.id.replace('addr-', '');
                    break;
                }
            }

            if (topAddr) {
                const bankHex = parseInt(this.currentBank).toString(16).toUpperCase().padStart(2, '0');
                const addrHex = parseInt(topAddr).toString(16).toUpperCase().padStart(4, '0');
                const newHash = `#bank-${bankHex}-addr-${addrHex}`;
                history.replaceState(null, null, newHash);
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
                if (v.endsWith(':')) v = v.slice(0, -1);
                return v;
            }
            if (field === 'comment' || field === 'block_comment') {
                return text.split('\n').map(line => {
                    let l = line.trimStart();
                    if (l.startsWith(';')) {
                        l = l.substring(1);
                        if (l.startsWith(' ')) l = l.substring(1);
                    }
                    return l;
                }).join('\n').trimEnd();
            }
            return text;
        }
    }));
});
