use super::regex::{HandlebarCapture, regex_handlebars};
use crate::blox::collection::Collection;
use crate::config::processor_config::Config;
use anyhow::Result;
use pathdiff::diff_paths;
use regex::Captures;
use std::path::PathBuf;

pub fn replace_refs(
    collection: &Collection,
    content: String,
    config: &Config,
    path: &PathBuf,
) -> Result<String> {
    let regex = regex_handlebars("blox(fig)?", "[ltnfTNF]?ref")?;

    let new_content = regex
        .replace_all(&content, |caps: &Captures| {
            let Some(refs) = HandlebarCapture::from_captures(caps) else {
                tracing::error!("Could not match handlebar");
                return HandlebarCapture::error("could not match");
            };

            if refs.keyword == "bloxfig" {
                replace_figure_ref(refs, collection, path)
            } else {
                replace_blox_ref(refs, collection, config, path)
            }
        })
        .to_string();

    Ok(new_content)
}

fn replace_blox_ref(
    refs: HandlebarCapture,
    collection: &Collection,
    config: &Config,
    path: &PathBuf,
) -> String {
    let Some(blox) = collection.bloxes().get(refs.label) else {
        return refs.to_error("blox not defined");
    };
    let Some(blox_label) = blox.blox_label() else {
        // Should never happen
        return refs.to_error("blox without label");
    };

    let ref_opts = refs.refs.strip_suffix("ref");

    match ref_opts {
        // Give title
        Some("T") => return blox.ref_title(),
        // Give number
        Some("N") => return blox.ref_number_unscoped(),
        // Give full
        Some("F") => return blox.ref_full(config),
        _ => (),
    };

    let path = relative_path_to_obj(path, blox_label.path(), blox.id(config));

    match ref_opts {
        // Give link
        Some("l") => path,
        // Provide linked environment-title
        Some("t") => markdown_link(&blox.ref_title(), &path),
        // Provide linked environment-number
        Some("n") => markdown_link(&blox.ref_number(config), &path),
        // Provide linked environment-number-title
        Some("f") => markdown_link(&blox.ref_full(config), &path),
        // Default
        _ => markdown_link(&blox.ref_default(config), &path),
    }
}

fn replace_figure_ref(refs: HandlebarCapture, collection: &Collection, path: &PathBuf) -> String {
    let Some(figure) = collection.figures().get(refs.label) else {
        return refs.to_error("figure not defined");
    };

    let ref_opts = refs.refs.strip_suffix("ref");

    match ref_opts {
        // Give number
        Some("N") => return figure.ref_number(),
        // Give full
        Some("F") => return figure.ref_default(),
        _ => (),
    };

    let path = relative_path_to_obj(path, figure.path(), figure.id());

    match ref_opts {
        // Give link
        Some("l") => path,
        // Default
        _ => markdown_link(&figure.ref_default(), &path),
    }
}

fn relative_path_to_obj(chapter_path: &PathBuf, obj_path: &PathBuf, id: Option<String>) -> String {
    path_to_obj(relative_path(chapter_path, obj_path), id)
}

fn path_to_obj(rel_path: String, id: Option<String>) -> String {
    id.map(|id_s| format!(r#"{rel_path}#{id_s}"#))
        .unwrap_or_default()
}

fn relative_path(chapter_path: &PathBuf, obj_path: &PathBuf) -> String {
    if chapter_path == obj_path {
        return String::new();
    }

    let mut local_chap_path = chapter_path.clone();
    local_chap_path.pop();

    diff_paths(obj_path, &local_chap_path)
        .unwrap()
        .display()
        .to_string()
}

fn markdown_link(text: &str, link: &str) -> String {
    format!("[{text}]({link})")
}
