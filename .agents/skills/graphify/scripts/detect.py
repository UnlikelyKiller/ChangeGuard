import json
import sys
import os
from pathlib import Path
from graphify.detect import detect, detect_incremental

def main():
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    incremental = "--update" in sys.argv
    
    if incremental:
        result = detect_incremental(target)
    else:
        result = detect(target)
        
    # Output raw JSON for the agent to read silently
    print(json.dumps(result))
    
    # Save to a temp file so the skill can read it if needed
    os.makedirs("graphify-out", exist_ok=True)
    with open("graphify-out/.graphify_detect.json", "w") as f:
        json.dump(result, f)

if __name__ == "__main__":
    main()
