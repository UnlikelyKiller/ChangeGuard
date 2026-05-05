import json
import os
from pathlib import Path
from datetime import datetime, timezone
from graphify.detect import save_manifest

def main():
    graphify_out = Path('graphify-out')
    detect_path = graphify_out / '.graphify_detect.json'
    extract_path = graphify_out / '.graphify_extract.json'
    
    if detect_path.exists():
        detect = json.loads(detect_path.read_text())
        save_manifest(detect['files'])
        
        if extract_path.exists():
            extract = json.loads(extract_path.read_text())
            input_tok = extract.get('input_tokens', 0)
            output_tok = extract.get('output_tokens', 0)
            
            cost_path = graphify_out / 'cost.json'
            if cost_path.exists():
                cost = json.loads(cost_path.read_text())
            else:
                cost = {'runs': [], 'total_input_tokens': 0, 'total_output_tokens': 0}
            
            cost['runs'].append({
                'date': datetime.now(timezone.utc).isoformat(),
                'input_tokens': input_tok,
                'output_tokens': output_tok,
                'files': detect.get('total_files', 0),
            })
            cost['total_input_tokens'] += input_tok
            cost['total_output_tokens'] += output_tok
            cost_path.write_text(json.dumps(cost, indent=2))
            
            print(f"This run: {input_tok:,} in / {output_tok:,} out")

    # Cleanup temp files
    temp_files = [
        '.graphify_detect.json', '.graphify_extract.json', '.graphify_ast.json',
        '.graphify_semantic.json', '.graphify_analysis.json', '.graphify_incremental.json'
    ]
    for f in temp_files:
        p = graphify_out / f
        if p.exists():
            p.unlink()
            
    # Remove chunks
    for f in os.listdir(graphify_out):
        if f.startswith('.graphify_chunk_'):
            (graphify_out / f).unlink()

if __name__ == "__main__":
    main()
