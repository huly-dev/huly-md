// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use generic_btree::rle::HasLength;
use lazy_static::lazy_static;
use loro_delta::DeltaItem;
use loro_internal::{
    container::ContainerID,
    delta::{Meta, StyleMeta},
    event::{Diff, DiffEvent, TextDiffItem},
    handler::TextDelta,
    ContainerType, LoroDoc, LoroValue,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

lazy_static! {
    static ref DOCS: Mutex<HashMap<String, Arc<LoroDoc>>> = Mutex::new(HashMap::new());
}

fn get_doc(doc_id: &str) -> Arc<LoroDoc> {
    let mut docs = DOCS.lock().unwrap();
    match docs.get(doc_id) {
        Some(doc) => Arc::clone(doc),
        None => {
            let mut doc = LoroDoc::new();
            doc.start_auto_commit();
            let arc = Arc::new(doc);
            docs.insert(doc_id.to_string(), Arc::clone(&arc));
            arc
        }
    }
}

#[tauri::command]
fn get_text_value(doc_id: &str, cid: &str) -> Result<LoroValue, String> {
    let container_id = ContainerID::try_from(cid).map_err(|_| "Invalid ContainerID")?;
    if let ContainerType::Text = container_id.container_type() {
        let doc = get_doc(doc_id);
        let handler = doc.get_text(container_id);
        let value = handler.get_richtext_value();
        Ok(value)
    } else {
        Err("ContainerID is not of type Text".into())
    }
}

#[tauri::command]
fn apply_delta(doc_id: &str, cid: &str, origin: &str, delta: Vec<TextDelta>) -> Result<(), String> {
    let container_id = ContainerID::try_from(cid).map_err(|_| "Invalid ContainerID")?;
    match container_id.container_type() {
        ContainerType::Text => {
            let doc = get_doc(doc_id);
            let vv = doc.state_vv();
            let handler = doc.get_text(container_id);
            handler
                .apply_delta(delta.as_slice())
                .map_err(|e| e.to_string())?;
            doc.commit_with(Some(origin.into()), None, true);
            let delta = doc.export_from(&vv);
            Ok(())
        }
        _ => Err("ContainerID is not of type Text".into()),
    }
}

#[derive(Serialize, Clone, Debug)]
enum HulyDiff {
    Text {
        #[serde(rename = "type")]
        kind: String,
        diff: Vec<HulyTextDiffItem>,
    },
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
enum HulyTextDiffItem {
    Retain {
        retain: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        attributes: Option<StyleMeta>,
    },
    Insert {
        insert: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        attributes: Option<StyleMeta>,
    },
    Delete {
        delete: usize,
    },
}

fn style(attr: &StyleMeta) -> Option<StyleMeta> {
    if attr.is_empty() {
        None
    } else {
        Some(attr.clone())
    }
}

fn text_diff_item_to_value(value: &TextDiffItem) -> Vec<HulyTextDiffItem> {
    match value {
        DeltaItem::Retain { len, attr } => vec![HulyTextDiffItem::Retain {
            retain: *len,
            attributes: style(attr),
        }],

        DeltaItem::Replace {
            value,
            attr,
            delete,
        } => {
            let mut result = Vec::new();
            if value.rle_len() > 0 {
                result.push(HulyTextDiffItem::Insert {
                    insert: value.to_string(),
                    attributes: style(attr),
                });
            }
            if *delete > 0 {
                result.push(HulyTextDiffItem::Delete { delete: *delete });
            }
            result
        }
    }
}

fn resolved_diff_to_value(value: &Diff) -> HulyDiff {
    match value {
        Diff::Text(text) => HulyDiff::Text {
            kind: "text".to_string(),
            diff: text.iter().map(text_diff_item_to_value).flatten().collect(),
        },
        _ => unreachable!(),
    }
}

#[tauri::command]
fn subscribe(app_handle: AppHandle, doc_id: &str, cid: &str) -> Result<u32, String> {
    dbg!(&doc_id);
    let app_handle_arc = Arc::new(app_handle);
    let handler = {
        let app_handle = Arc::clone(&app_handle_arc);
        Arc::new(move |e: DiffEvent| {
            dbg!(&e);
            for event in e.events {
                let value = resolved_diff_to_value(&event.diff);
                // dbg!(&value);
                app_handle.emit("diff", value);
            }
        })
    };
    let container_id = ContainerID::try_from(cid).map_err(|_| "Invalid ContainerID")?;
    let doc = get_doc(doc_id);
    let sub_id = doc.subscribe(&container_id, handler);
    Ok(sub_id.into_u32())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_text_value,
            apply_delta,
            subscribe
        ])
        .run(tauri::generate_context!())
        .expect("error while running Huly MD");
}
