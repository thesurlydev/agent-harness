You are implementing a module for a Rust project using ast-crdt intents.

## Current Source
<<SOURCE>>

## Task
<<TASK_DESCRIPTION>>

## Available Intent Types

### add_use
```json
{"type":"add_use","path":"std::collections::HashMap","file":"src/lib.rs"}
```

### add_struct
```json
{"type":"add_struct","name":"Config","visibility":"pub","attributes":["#[derive(Debug, Clone)]"],"fields":[{"name":"host","type":"String","visibility":"pub"},{"name":"port","type":"u16","visibility":"pub"}],"file":"src/types.rs"}
```

### add_enum
```json
{"type":"add_enum","name":"Status","visibility":"pub","variants":["Pending","Running","Done(String)"],"attributes":["#[derive(Debug)]"],"file":"src/types.rs"}
```

### add_function
```json
{"type":"add_function","name":"process","visibility":"pub","is_async":false,"params":[["input","&str"],["count","usize"]],"returns":"String","body":"{ input.repeat(count) }","file":"src/lib.rs"}
```

### add_impl
```json
{"type":"add_impl","target_type":"Config","file":"src/types.rs"}
```

### add_method (with self receiver)
```json
{"type":"add_method","impl_target":"Store","name":"get","visibility":"pub","receiver":"&self","params":[["id","u64"]],"returns":"Option<Item>","body":"{ self.items.get(&id).cloned() }","file":"src/store.rs"}
```

### add_method (static / constructor, no receiver)
```json
{"type":"add_method","impl_target":"Store","name":"new","visibility":"pub","params":[],"returns":"Self","body":"{ Self { items: HashMap::new() } }","file":"src/store.rs"}
```

### set_function_body
```json
{"type":"set_function_body","fn_name":"main","body":"{ println!(\"hello\"); }","file":"src/main.rs"}
```

### add_attribute
```json
{"type":"add_attribute","target":"main","kind":"function","attribute":"#[tokio::main]","file":"src/main.rs"}
```

## Rules
- Output ONLY a JSON array, no markdown, no explanation
- Each element is: {"intent": {<intent object>}}
- Every intent MUST include the "file" field specifying the target source file
- Function/method bodies must be valid Rust wrapped in { }
- Methods with parameters MUST include the "params" field as [["name","Type"],...] — omitting params produces methods with no arguments
- Static methods (constructors) omit "receiver"; instance methods use "receiver":"&self" or "&mut self"
- Use std::sync::Mutex with .lock().unwrap() for simple in-memory state
- Use axum::extract::{State, Path, Json} for handler extractors
- Do NOT add explicit add_use intents for standard library types, serde derives, or types from sibling project files — the orchestrator inserts these imports automatically
- DO add add_use for less common external crate paths that aren't auto-resolved
