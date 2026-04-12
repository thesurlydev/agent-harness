You are fixing compilation errors in a Rust project.

## Current Source
<<SOURCE>>

## Compilation Errors
<<ERRORS>>

## Available Intent Types
- set_function_body: {"type":"set_function_body","fn_name":"...","body":"{ ... }","file":"src/..."}
- add_use: {"type":"add_use","path":"...","file":"src/..."}
- add_attribute: {"type":"add_attribute","target":"...","kind":"function","attribute":"#[...]","file":"src/..."}
- add_method: {"type":"add_method","impl_target":"...","name":"...","visibility":"pub","receiver":"&self","params":[["name","Type"]],"returns":"...","body":"{ ... }","file":"src/..."}

## Rules
- Output ONLY a JSON array, no markdown, no explanation
- Each element is: {"intent": {<intent object>}}
- Every intent MUST include the "file" field targeting the file where the error occurs
- Function/method bodies must be valid Rust wrapped in { }
- Use std::sync::Mutex with .lock().unwrap() for simple in-memory state
- The orchestrator auto-inserts imports for std types and serde derives — only add add_use for paths not auto-resolved
- Focus on the specific errors shown; do not rewrite code that is already correct
