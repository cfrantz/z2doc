document.addEventListener('alpine:init', () => {
    Alpine.data('disasmApp', () => ({
        metadata: { rom_file: '', total_banks: 0, mapper_window_size: 0 },
        currentBank: 0,
        disassembly: [],
        themes: [],
        currentTheme: 'Dark',

        async init() {
            await this.fetchMetadata();
            await this.fetchThemes();
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
            // Scroll to top when changing banks
            window.scrollTo(0, 0);
        },

        async updateAnnotation(line, field, value) {
            const processedValue = (field === 'comment' || field === 'block_comment') 
                ? this.stripSemicolons(value) 
                : value;

            // Only update if value actually changed to avoid redundant saves
            if (line[field] === processedValue || (line[field] === null && processedValue === "")) return;

            const req = {
                bank_id: field === 'symbol' && line.bank === null ? null : parseInt(this.currentBank),
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
                // Update local state to avoid full refresh if possible, 
                // but re-fetching ensures all references are updated.
                await this.fetchDisassembly();
            }
        },

        navigate(targetBank, targetAddress) {
            if (targetBank !== null && targetBank !== this.currentBank) {
                this.currentBank = targetBank;
                this.fetchDisassembly();
            }
            // Scrolling to the specific address could be implemented by ID
            const element = document.getElementById(`addr-${targetAddress}`);
            if (element) {
                element.scrollIntoView();
            }
        },

        stripSemicolons(text) {
            if (!text) return text;
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
    }));
});
