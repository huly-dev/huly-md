// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use generic_btree::rle::HasLength;
use lazy_static::lazy_static;
use loro_delta::DeltaItem;
use loro_internal::{
    configure::{StyleConfig, StyleConfigMap},
    container::{richtext::ExpandType, ContainerID},
    delta::{Meta, StyleMeta},
    event::{ContainerDiff, Diff, DiffEvent, DocDiff, TextDiffItem},
    handler::TextDelta,
    obs::SubID,
    ContainerType, LoroDoc, LoroValue, ToJson,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

struct Loro {
    doc: Arc<LoroDoc>,
    sub: SubID,
}

lazy_static! {
    static ref DOCS: Mutex<HashMap<String, Loro>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Clone, Debug)]
struct HulyDocDiff {
    origin: String,
    #[serde(rename = "docId")]
    doc_id: String,
    diff: Vec<HulyContainerDiff>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
enum HulyContainerDiff {
    Text {
        id: String,
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
        attributes: Option<Value>,
    },
    Insert {
        insert: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        attributes: Option<Value>,
    },
    Delete {
        delete: usize,
    },
}

fn hulyize_doc_diff(doc_id: String, doc_diff: &DocDiff) -> HulyDocDiff {
    HulyDocDiff {
        doc_id,
        origin: doc_diff.origin.to_string(),
        diff: doc_diff.diff.iter().map(hulyize_container_diff).collect(),
    }
}

fn style(attr: &StyleMeta) -> Option<Value> {
    if attr.is_empty() {
        None
    } else {
        Some(attr.to_json_value())
    }
}

fn hulyize_text_diff_item(value: &TextDiffItem) -> Vec<HulyTextDiffItem> {
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
            let mut result = Vec::with_capacity(2);
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

fn hulyize_container_diff(value: &ContainerDiff) -> HulyContainerDiff {
    match value.id.container_type() {
        ContainerType::Text => {
            if let Diff::Text(text_diff) = &value.diff {
                HulyContainerDiff::Text {
                    id: value.id.name().to_string(),
                    kind: "text".to_string(),
                    diff: text_diff
                        .iter()
                        .map(hulyize_text_diff_item)
                        .flatten()
                        .collect(),
                }
            } else {
                unreachable!()
            }
        }
        _ => unreachable!(),
    }
}

fn config_text_style() -> StyleConfigMap {
    let mut style_config = StyleConfigMap::new();
    style_config.insert(
        "bold".into(),
        StyleConfig {
            expand: ExpandType::After,
        },
    );
    style_config.insert(
        "italic".into(),
        StyleConfig {
            expand: ExpandType::After,
        },
    );
    style_config.insert(
        "list".into(),
        StyleConfig {
            expand: ExpandType::After,
        },
    );
    style_config.insert(
        "indent".into(),
        StyleConfig {
            expand: ExpandType::After,
        },
    );
    style_config.insert(
        "link".into(),
        StyleConfig {
            expand: ExpandType::After,
        },
    );
    style_config
}

fn get_doc(app_handle: Arc<AppHandle>, doc_id: &str) -> Arc<LoroDoc> {
    let mut docs = DOCS.lock().unwrap();
    match docs.get(doc_id) {
        Some(loro) => Arc::clone(&loro.doc),
        None => {
            let doc = Arc::new(LoroDoc::new_auto_commit());
            let id = doc_id.to_string();

            doc.config_text_style(config_text_style());

            let handler = {
                let app_handle = Arc::clone(&app_handle);
                Arc::new(move |e: DiffEvent| {
                    dbg!(&e);
                    let value = hulyize_doc_diff(id.clone(), e.event_meta);
                    dbg!(&value);
                    app_handle.emit("doc-diff", value);
                })
            };

            let loro = Loro {
                doc: Arc::clone(&doc),
                sub: doc.subscribe_root(handler),
            };
            docs.insert(doc_id.to_string(), loro);
            doc
        }
    }
}

#[tauri::command]
fn get_text_value(app_handle: AppHandle, doc_id: &str, path: &str) -> Result<LoroValue, String> {
    let doc = get_doc(Arc::new(app_handle), doc_id);
    let handler = doc.get_text(ContainerID::new_root(path, ContainerType::Text));
    Ok(handler.get_richtext_value())
}

#[tauri::command]
fn apply_delta(
    app_handle: AppHandle,
    doc_id: &str,
    path: &str,
    origin: &str,
    delta: Vec<TextDelta>,
) -> Result<Vec<u8>, String> {
    let doc = get_doc(Arc::new(app_handle), doc_id);
    let vv = doc.state_vv();
    let handler = doc.get_text(ContainerID::new_root(path, ContainerType::Text));
    handler
        .apply_delta(delta.as_slice())
        .map_err(|e| e.to_string())?;
    doc.commit_with(Some(origin.into()), None, true);
    Ok(doc.export_from(&vv))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_text_value, apply_delta])
        .run(tauri::generate_context!())
        .expect("error while running Huly MD");
}
