You are implementing a module for a Rust project using ast-crdt intents.

## Current Source
<<SOURCE>>

## Task
<<TASK_DESCRIPTION>>

## Available Intent Types
- add_use: {"type":"add_use","path":"...","file":"..."}
- add_struct: {"type":"add_struct","name":"...","visibility":"pub","attributes":["#[derive(...)]"],"fields":[{"name":"...","type":"...","visibility":"pub"}],"file":"..."}
- add_enum: {"type":"add_enum","name":"...","visibility":"pub","variants":["A","B"],"attributes":[],"file":"..."}
- add_function: {"type":"add_function","name":"...","visibility":"pub","is_async":false,"params":[["arg","Type"]],"returns":"RetType","body":"{ ... }","file":"..."}
- add_impl: {"type":"add_impl","target_type":"...","trait_name":null,"file":"..."}
- add_method: {"type":"add_method","impl_target":"...","impl_trait":null,"name":"...","visibility":"pub","receiver":"&self","returns":"...","body":"{ ... }","is_async":false,"file":"..."}
- set_function_body: {"type":"set_function_body","fn_name":"...","body":"{ ... }","file":"..."}
- add_attribute: {"type":"add_attribute","target":"...","kind":"function","attribute":"#[tokio::main]","file":"..."}

## Rules
- Output ONLY a JSON array, no markdown, no explanation
- Each element is: {"intent": {<intent object>}}
- Function bodies must be valid Rust wrapped in { }
- Use tokio::sync::Mutex (not std::sync::Mutex) for Axum shared state
- Use {param} route syntax for Axum 0.8
