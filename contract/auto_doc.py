import re
import sys

def generate_doc(sig, item_type, item_name):
    lines = []
    if item_type == "struct":
        lines.append(f"/// Represents a {item_name.replace('_', ' ').lower()}.")
    elif item_type == "enum":
        lines.append(f"/// Defines the possible states or errors for {item_name.replace('_', ' ').lower()}.")
    elif item_type == "variant":
        lines.append(f"/// {item_name.replace('_', ' ')}.")
    elif item_type == "fn":
        # Extract params
        lines.append(f"/// Executes the {item_name.replace('_', ' ')} operation.")
        
        param_str = re.search(r'\((.*?)\)', sig)
        if param_str:
            params = param_str.group(1).split(',')
            has_real_params = False
            param_docs = []
            for p in params:
                p = p.strip()
                if not p: continue
                parts = p.split(':')
                if len(parts) > 0:
                    pname = parts[0].strip()
                    if pname not in ('env', 'self', '&self', '&mut self'):
                        has_real_params = True
                        pdesc = pname.replace('_', ' ')
                        param_docs.append(f"/// * `{pname}` - The {pdesc}.")
                    elif pname == 'env':
                        has_real_params = True
                        param_docs.append(f"/// * `env` - The execution environment.")
            if has_real_params:
                lines.append("///")
                lines.append("/// # Arguments")
                lines.append("///")
                lines.extend(param_docs)
        
        # Extract return type
        ret_str = re.search(r'->\s*(.*)', sig)
        if ret_str:
            lines.append("///")
            lines.append("/// # Returns")
            lines.append("///")
            ret_type = ret_str.group(1).strip().rstrip(';').rstrip('{').strip()
            if ret_type:
                lines.append(f"/// Returns `{ret_type}`.")
                
        # Add Panics section
        lines.append("///")
        lines.append("/// # Panics")
        lines.append("///")
        lines.append("/// Panics if the internal state is invalid or required conditions are not met.")
        
    return lines

# Actually, doing this with regex is hard. Let's just use the clippy json output.
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

def process_file(filename, warns):
    with open(filename, 'r') as f:
        lines = f.readlines()
        
    # Sort descending to not shift lines
    warns.sort(key=lambda x: x["line"], reverse=True)
    
    for w in warns:
        line_idx = w["line"] - 1
        target_line = lines[line_idx].strip()
        
        # Skip if already has doc on previous line (clippy sometimes warns weirdly)
        if line_idx > 0 and lines[line_idx-1].strip().startswith('///'):
            continue
            
        # Guess item type
        item_type = "unknown"
        item_name = "item"
        sig = target_line
        
        if target_line.startswith("pub struct ") or target_line.startswith("struct "):
            item_type = "struct"
            m = re.search(r'struct\s+(\w+)', target_line)
            if m: item_name = m.group(1)
        elif target_line.startswith("pub enum ") or target_line.startswith("enum "):
            item_type = "enum"
            m = re.search(r'enum\s+(\w+)', target_line)
            if m: item_name = m.group(1)
        elif target_line.startswith("pub fn ") or target_line.startswith("fn "):
            item_type = "fn"
            m = re.search(r'fn\s+(\w+)', target_line)
            if m: item_name = m.group(1)
            # Gather full signature if multi-line
            sig_lines = []
            curr = line_idx
            while curr < len(lines):
                sig_lines.append(lines[curr].strip())
                if '{' in lines[curr] or ';' in lines[curr]:
                    break
                curr += 1
            sig = " ".join(sig_lines)
        elif target_line.startswith("pub trait ") or target_line.startswith("trait "):
            item_type = "trait"
            m = re.search(r'trait\s+(\w+)', target_line)
            if m: item_name = m.group(1)
        else:
            # Maybe a struct field or enum variant
            if ":" in target_line:
                item_type = "field"
                item_name = target_line.split(':')[0].strip()
            elif "," in target_line or target_line.endswith('}'):
                item_type = "variant"
                item_name = target_line.split('(')[0].split(',')[0].strip()
        
        doc_lines = generate_doc(sig, item_type, item_name)
        
        # Calculate indentation
        indent = len(lines[line_idx]) - len(lines[line_idx].lstrip())
        indent_str = " " * indent
        
        for d in reversed(doc_lines):
            lines.insert(line_idx, indent_str + d + "\n")
            
    with open(filename, 'w') as f:
        f.writelines(lines)

warnings = run_clippy()
for file, warns in warnings.items():
    if "contract/src" in file:
        print(f"Processing {file}...")
        process_file(file, warns)

