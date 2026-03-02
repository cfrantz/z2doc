import json
import re
import os

def to_snake_case(name):
    # Handle PascalCase/camelCase by inserting underscores before capitals
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower().replace('__', '_')

def main():
    db_path = 'zelda2.json'
    if not os.path.exists(db_path):
        print(f"Error: {db_path} not found.")
        return

    with open(db_path, 'r') as f:
        data = json.load(f)

    if 'bank' in data:
        for bank_id, bank_info in data['bank'].items():
            prefix = f"bank{bank_id}_"
            if 'address' in bank_info:
                for addr, anno in bank_info['address'].items():
                    if 'symbol' in anno and anno['symbol']:
                        original = anno['symbol']
                        
                        # Aggressively strip ALL leading bankN_ prefixes
                        clean_original = original
                        while re.match(r'^bank\d+_', clean_original, re.IGNORECASE):
                            clean_original = re.sub(r'^bank\d+_', '', clean_original, count=1, flags=re.IGNORECASE)
                        
                        snake = to_snake_case(clean_original)
                        new_symbol = f"{prefix}{snake}"
                        anno['symbol'] = new_symbol
                        if original != new_symbol:
                            print(f"Bank {bank_id} at {addr}: {original} -> {new_symbol}")

    with open(db_path, 'w') as f:
        json.dump(data, f, indent=2)
    
    print("Update complete.")

if __name__ == "__main__":
    main()
