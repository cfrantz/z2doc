import os
import re
import json
import glob

# Constants
DB_PATH = 'zelda2.json'
JUNK_DIR = 'junk'

# Regex patterns
RE_SEGMENT = re.compile(r'\.segment\s+"PRG(\d+)"')
RE_LABEL = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*):')
RE_ADDR_COMMENT = re.compile(r';\s*0x[0-9A-Fa-f]+\s+\$([0-9A-Fa-f]{4})[^;]*')
RE_AUTO_LABEL = re.compile(r'^L[0-9A-Fa-f]{4}$')
RE_BORDER = re.compile(r'^;\s*-+\s*;?$')

# Useless comment patterns: A = 18, X -> 17, etc.
RE_USELESS_COMMENT = re.compile(r'^[A-Z]\s*(=|->)\s*[0-9A-F]{2}$', re.IGNORECASE)

def clean_comment(text):
    """Strip leading/trailing semicolons and whitespace."""
    t = text.strip()
    # Strip leading semicolons
    while t.startswith(';'):
        t = t[1:].strip()
    # Strip trailing semicolons
    while t.endswith(';'):
        t = t[:-1].strip()
    return t

def is_useless_comment(text):
    """Filter out patterns like 'A = 18' or 'X -> 17'."""
    return bool(RE_USELESS_COMMENT.match(text))

def extract_metadata():
    if not os.path.exists(DB_PATH):
        print(f"Error: {DB_PATH} not found.")
        return

    with open(DB_PATH, 'r') as f:
        db = json.load(f)

    new_metadata = {}

    asm_files = glob.glob(os.path.join(JUNK_DIR, '*.asm'))
    for asm_path in asm_files:
        print(f"Processing {asm_path}...")
        with open(asm_path, 'r', encoding='utf-8', errors='ignore') as f:
            current_bank = None
            pending_label = None
            pending_block_comment = []
            
            for line in f:
                raw_line = line.rstrip()
                line = raw_line.strip()
                if not line:
                    continue

                if RE_SEGMENT.search(line):
                    seg_match = RE_SEGMENT.search(line)
                    current_bank = int(seg_match.group(1))
                    if current_bank not in new_metadata:
                        new_metadata[current_bank] = {}
                    continue

                if RE_BORDER.match(line):
                    continue

                if line.startswith(';'):
                    addr_match = RE_ADDR_COMMENT.search(line)
                    if not addr_match:
                        comment_text = clean_comment(line[1:])
                        if comment_text and not is_useless_comment(comment_text):
                            pending_block_comment.append(comment_text)
                        continue

                label_match = RE_LABEL.match(line)
                if label_match:
                    label_name = label_match.group(1)
                    if not RE_AUTO_LABEL.match(label_name):
                        pending_label = label_name
                
                addr_match = RE_ADDR_COMMENT.search(raw_line)
                if addr_match and current_bank is not None:
                    address = int(addr_match.group(1), 16)
                    
                    entry = {}
                    if pending_label:
                        entry['symbol'] = pending_label
                    if pending_block_comment:
                        entry['block_comment'] = "\n".join(pending_block_comment)
                    
                    comment_parts = raw_line.split(';')
                    if len(comment_parts) > 2:
                        english_comment = clean_comment(";".join(comment_parts[2:]))
                        if english_comment and not is_useless_comment(english_comment):
                            entry['comment'] = english_comment
                    
                    if entry:
                        if address not in new_metadata[current_bank]:
                            new_metadata[current_bank][address] = entry
                        else:
                            if 'symbol' in entry:
                                new_metadata[current_bank][address]['symbol'] = entry['symbol']
                            if 'comment' in entry:
                                new_metadata[current_bank][address]['comment'] = entry['comment']
                            if 'block_comment' in entry:
                                existing_bc = new_metadata[current_bank][address].get('block_comment', '')
                                if existing_bc:
                                    new_metadata[current_bank][address]['block_comment'] = existing_bc + "\n" + entry['block_comment']
                                else:
                                    new_metadata[current_bank][address]['block_comment'] = entry['block_comment']

                    pending_label = None
                    pending_block_comment = []

    for bank_id, addresses in new_metadata.items():
        bank_key = str(bank_id)
        if bank_key not in db['bank']:
            db['bank'][bank_key] = {
                "region": [],
                "address": {},
                "mapped_at": 0x8000 if bank_id < 7 else 0xC000
            }
        
        for addr, meta in addresses.items():
            addr_key = str(addr)
            if addr_key not in db['bank'][bank_key]['address']:
                db['bank'][bank_key]['address'][addr_key] = {}
            
            existing = db['bank'][bank_key]['address'][addr_key]
            if 'symbol' in meta:
                existing['symbol'] = meta['symbol']
            if 'comment' in meta:
                existing['comment'] = meta['comment']
            if 'block_comment' in meta:
                existing['block_comment'] = meta['block_comment']

    with open(DB_PATH, 'w') as f:
        json.dump(db, f, indent=2)
    
    print(f"Done. Successfully merged metadata into {DB_PATH}")

if __name__ == "__main__":
    extract_metadata()
