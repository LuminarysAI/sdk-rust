//! Helpers for building [`Command`] values.

use crate::types::{BatchItem, Command, CommandType};

/// Publish an event to the MessageBus.
pub fn emit_event(topic: impl Into<String>, payload: rmpv::Value) -> Command {
    Command {
        kind: CommandType::EmitEvent,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("topic".into()), rmpv::Value::String(topic.into().into())),
            (rmpv::Value::String("payload".into()), payload),
        ])),
    }
}

/// Invoke another skill via `call_module`.
pub fn call_module(
    skill_id: impl Into<String>,
    method: impl Into<String>,
    payload: rmpv::Value,
    callback: impl Into<String>,
    call_ctx: impl Into<String>,
) -> Command {
    Command {
        kind: CommandType::CallModule,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("skill_id".into()), rmpv::Value::String(skill_id.into().into())),
            (rmpv::Value::String("method".into()), rmpv::Value::String(method.into().into())),
            (rmpv::Value::String("payload".into()), payload),
            (rmpv::Value::String("callback".into()), rmpv::Value::String(callback.into().into())),
            (rmpv::Value::String("call_ctx".into()), rmpv::Value::String(call_ctx.into().into())),
        ])),
    }
}

/// Store a value in the Shared KV (L3).
pub fn store_kv(key: impl Into<String>, value: rmpv::Value) -> Command {
    Command {
        kind: CommandType::StoreKv,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("key".into()), rmpv::Value::String(key.into().into())),
            (rmpv::Value::String("value".into()), value),
        ])),
    }
}

/// Load a value from the Shared KV (L3). Result delivered to `callback`.
pub fn load_kv(key: impl Into<String>, callback: impl Into<String>) -> Command {
    Command {
        kind: CommandType::LoadKv,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("key".into()), rmpv::Value::String(key.into().into())),
            (rmpv::Value::String("callback".into()), rmpv::Value::String(callback.into().into())),
        ])),
    }
}

/// Schedule a delayed invocation of `method` in this skill.
pub fn schedule(method: impl Into<String>, delay_ms: i64, payload: rmpv::Value) -> Command {
    Command {
        kind: CommandType::Schedule,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("method".into()), rmpv::Value::String(method.into().into())),
            (rmpv::Value::String("delay_ms".into()), rmpv::Value::Integer(delay_ms.into())),
            (rmpv::Value::String("payload".into()), payload),
        ])),
    }
}

/// Build a `batch_invoke` command.
pub fn batch_invoke(items: Vec<BatchItem>, callback: impl Into<String>, concurrency: u32) -> Command {
    let raw_items: Vec<rmpv::Value> = items
        .into_iter()
        .map(|it| {
            rmpv::Value::Map(vec![
                (rmpv::Value::String("index".into()), rmpv::Value::Integer(it.index.into())),
                (rmpv::Value::String("skill_id".into()), rmpv::Value::String(it.skill_id.into())),
                (rmpv::Value::String("method".into()), rmpv::Value::String(it.method.into())),
                (rmpv::Value::String("payload".into()), rmpv::Value::Binary(it.payload)),
            ])
        })
        .collect();

    Command {
        kind: CommandType::BatchInvoke,
        payload: Some(rmpv::Value::Map(vec![
            (rmpv::Value::String("items".into()), rmpv::Value::Array(raw_items)),
            (rmpv::Value::String("callback".into()), rmpv::Value::String(callback.into().into())),
            (rmpv::Value::String("concurrency".into()), rmpv::Value::Integer(concurrency.into())),
        ])),
    }
}
