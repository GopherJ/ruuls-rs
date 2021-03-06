# ruuls-rs

## Installation

Add this package to `Cargo.toml` of your project. (Check https://crates.io/crates/json-rules-engine for right version)

```toml
[dependencies]
json-rules-engine = { version = "0.2.0" }
tokio = { version = "0.3.3", features = ["macros"] }
serde_json = { version = "*" }
```

## Features

- Built in operators
- Full support for ALL and ANY boolean operators, including recursive nesting
- Type Safe
- Lightweight
- Load rules from json
- HTTP post to callback url
- Built in Moustache render

## Get started

```rust
use ruuls::{Engine, Rule};
use serde_json::json;

#[tokio::main]
async main() -> anyhow::Result<()> {
    let rule_json = json!({
        "conditions": {
            "and": [
                {
                    "field": "name",
                    "operator": "string_equals",
                    "value": "Cheng JIANG"
                },
                {
                    "field": "age",
                    "operator": "int_in_range",
                    "value": [20, 25] 
                },
                {
                    "field": "action",
                    "operator": "string_equals",
                    "value": "coding in rust"
                }
            ]
        },
        "event": {
            "type": "post_to_callback_url",
            "params": {
                "callback_url": "http://example.com/people/conding_in_rust"
                "type": "info",
                "title": "Another person is coding in rust",
                "message": "Name: {{ name }}, Age: {{ age }}, Action: {{ action }},"
            }
        }
    });

    let rule: Rule = serde_json::from_str(serde_json::to_string(&rules_json).unwrap()).unwrap();

    let mut engine = Engine::new();
    engine.add_rule(rule);

    let facts = json!({
        "name": "Cheng JIANG",
        "age": 24,
        "action": "coding in rust",
    });

    let rule_results = engine.run(&facts).await?;

    println!("{:?}", rule_results);
}
```


