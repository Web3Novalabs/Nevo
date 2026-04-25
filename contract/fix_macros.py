import os

for root, _, files in os.walk("contract/src"):
    for file in files:
        if file.endswith(".rs") and file != "test_doc.rs":
            filepath = os.path.join(root, file)
            with open(filepath, 'r') as f:
                lines = f.readlines()
            
            new_lines = []
            for line in lines:
                if "#[contracttype]" in line or "#[contractimpl]" in line or "#[contracterror]" in line or "#[contract]" in line:
                    indent = len(line) - len(line.lstrip())
                    new_lines.append(" " * indent + "#[allow(missing_docs)]\n")
                new_lines.append(line)
                
            with open(filepath, 'w') as f:
                f.writelines(new_lines)

