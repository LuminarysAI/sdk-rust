# luminarys-sdk

Rust SDK for building Luminarys WASM skills.

## Installation

Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
luminarys-sdk = "0.2"
serde = { version = "1", features = ["derive"] }
rmp-serde = "1.3"
```

## Quick Start

Create `src/skill.rs` with annotated handler functions (use `///` doc comments):

```rust
use luminarys_sdk::prelude::*;

/// @skill:id      com.my-company.my-skill
/// @skill:name    "My Skill"
/// @skill:version 1.0.0
/// @skill:desc    "My first skill."

/// @skill:method greet "Greet by name."
/// @skill:param  name required "User name"
/// @skill:result "Greeting text"
pub fn greet(_ctx: &mut Context, name: String) -> Result<String, SkillError> {
    Ok(format!("Hello, {}!", name))
}
```

Generate, build, and sign:

```bash
lmsk genkey                            # once: create developer signing key
lmsk generate -lang rust ./src        # generate src/lib.rs
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/my_skill.wasm  # → com.my-company.my-skill.skill
```

## Documentation

[luminarys.ai](https://luminarys.ai)

## Tools

Download `lmsk` from [releases](https://github.com/LuminarysAI/luminarys/releases).

## License

MIT
