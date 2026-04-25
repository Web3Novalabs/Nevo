import re
import sys
import json
import subprocess
from collections import defaultdict

def run_clippy():
    cmd = ["cargo", "clippy", "--message-format=json", "--", "-W", "missing_docs"]
    result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    warnings = defaultdict(list)
    for line in result.stdout.splitlines():
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and msg["message"]["code"] and msg["message"]["code"]["code"] == "missing_docs":
                for span in msg["message"]["spans"]:
                    if span["is_primary"]:
                        warnings[span["file_name"]].append({
                            "line": span["line_start"],
                            "text": span["text"][0]["text"] if span["text"] else ""
                        })
        except:
            pass
    return warnings

def generate_doc(line_str):
    name = "item"
    if "fn " in line_str:
        m = re.search(r'fn\s+(\w+)', line_str)
        if m: name = m.group(1)
        return [f"/// Executes {name.replace('_', ' ')}."]
    elif "pub " in line_str or ":" in line_str or "," in line_str:
        # field or variant
        parts = re.split(r'[:\(,]', line_str.strip())
        if parts:
            name = parts[0].replace('pub ', '').strip()
        return [f"/// The {name.replace('_', ' ')}."]
    return ["/// Documentation for this item."]

warnings = run_clippy()
for file, warns in warnings.items():
    if "contract/src" not in file:
        continue
    print(f"Fixing {file}")
    with open(file, 'r') as f:
        lines = f.readlines()
        
    warns.sort(key=lambda x: x["line"], reverse=True)
    
    for w in warns:
        lidx = w["line"] - 1
        line_str = lines[lidx]
        if line_str.strip().startswith("///"):
            continue
        if line_str.strip().startswith("#["):
            # If it's an attribute macro, the warning often points to it.
            # We want to put doc comment BEFORE the macro!
            # Wait, doc comments should go before #[contracttype] etc.
            pass
            
        doc = generate_doc(line_str)
        indent = len(line_str) - len(line_str.lstrip())
        indent_str = " " * indent
        
        for d in reversed(doc):
            lines.insert(lidx, indent_str + d + "\n")
            
    with open(file, 'w') as f:
        f.writelines(lines)
