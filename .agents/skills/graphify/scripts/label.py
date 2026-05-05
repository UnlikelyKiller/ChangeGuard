import json
import sys
from pathlib import Path
from graphify.build import build_from_json
from graphify.analyze import suggest_questions
from graphify.report import generate

def main():
    target_path = sys.argv[1] if len(sys.argv) > 1 else "."
    labels_file = Path('graphify-out/.graphify_labels.json')
    
    if not labels_file.exists():
        print("Error: labels missing")
        return

    extraction = json.loads(Path('graphify-out/.graphify_extract.json').read_text())
    detection  = json.loads(Path('graphify-out/.graphify_detect.json').read_text())
    analysis   = json.loads(Path('graphify-out/.graphify_analysis.json').read_text())

    G = build_from_json(extraction)
    communities = {int(k): v for k, v in analysis['communities'].items()}
    cohesion = {int(k): v for k, v in analysis['cohesion'].items()}
    tokens = {'input': extraction.get('input_tokens', 0), 'output': extraction.get('output_tokens', 0)}
    
    labels = {int(k): v for k, v in json.loads(labels_file.read_text()).items()}
    questions = suggest_questions(G, communities, labels)

    report = generate(G, communities, cohesion, labels, analysis['gods'], analysis['surprises'], detection, tokens, target_path, suggested_questions=questions)
    Path('graphify-out/GRAPH_REPORT.md').write_text(report)
    print("Report updated with community labels")

if __name__ == "__main__":
    main()
