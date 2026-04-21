# Track 40: Narrative Refinement

## Objective
Close the Track 34 audit gaps from `docs/audit4.md`: narrative mode must use one deterministic structured prompt, and fallback artifact failures must be visible.

## Requirements
- Do not nest a generated narrative prompt inside the generic `Question:` field.
- In narrative mode, build one flat structured prompt from the impact packet.
- Surface fallback artifact directory, serialization, and write failures instead of ignoring them.
- Preserve the exact missing Gemini CLI message.
- Keep deterministic narrative prompt tests green.
