import json
import subprocess
import sys
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
                        file_name = span["file_name"]
                        line_start = span["line_start"]
                        warnings[file_name].append({
                            "line": line_start,
                            "text": span["text"][0]["text"] if span["text"] else ""
                        })
        except json.JSONDecodeError:
            pass
    return warnings

if __name__ == "__main__":
    warnings = run_clippy()
    for file, warns in warnings.items():
        print(f"File: {file} ({len(warns)} warnings)")
        # Sort descending so we can insert without messing up line numbers
        for w in sorted(warns, key=lambda x: x["line"], reverse=True):
            pass
