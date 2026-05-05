import json
import sys
from pathlib import Path
from graphify.extract import collect_files, extract

def main():
    detect_path = Path("graphify-out/.graphify_detect.json")
    if not detect_path.exists():
        print("Error: detect result not found")
        sys.exit(1)
        
    with open(detect_path) as f:
        detect_data = json.load(f)
        
    code_files = []
    for f in detect_data.get('files', {}).get('code', []):
        p = Path(f)
        code_files.extend(collect_files(p) if p.is_dir() else [p])
        
    if code_files:
        result = extract(code_files, cache_root=Path('.'))
        with open("graphify-out/.graphify_ast.json", "w") as f:
            json.dump(result, f, indent=2)
        print(f"AST: {len(result['nodes'])} nodes, {len(result['edges'])} edges")
    else:
        with open("graphify-out/.graphify_ast.json", "w") as f:
            json.dump({'nodes':[], 'edges':[], 'input_tokens':0, 'output_tokens':0}, f)
        print("No code files - skipping AST extraction")

if __name__ == "__main__":
    main()
