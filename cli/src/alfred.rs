use std::fs;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use indexmap::indexmap;

use anyhow::{Context, Result};

pub struct WorkflowInfo {
    pub bundle_id: String,
    pub name: String,
    pub bin_name: String,
    pub author: String,
    pub description: String,
    pub keyword: String,
}

macro_rules! dict {
    ($($key:expr => $value:expr),*) => {
        plist::Value::Dictionary(
            indexmap!{$($key.clone().into() => $value.clone().into()),*}.into_iter().collect()
        )
    }
}

/// Builds an Alfred workflow `info.plist` file.
///
/// This is just a simple script filter to clipboard workflow.
pub fn build_info_plist(info: &WorkflowInfo) -> plist::Value {
    let uid_a = uuid::Uuid::new_v4().to_string().to_uppercase();
    let uid_b = uuid::Uuid::new_v4().to_string().to_uppercase();
    dict! {
        "name" => info.name,
        "description" => info.description,
        "bundleid" => info.bundle_id,
        "createdby" => info.author,
        "connections" => dict! {
            uid_a => vec![
                dict! { "destinationuid" => uid_b }
            ]
        },
        "uidata" => dict! {
            uid_a => dict! {
                "xpos" => 50,
                "ypos" => 50
            },
            uid_b => dict! {
                "xpos" => 225,
                "ypos" => 50
            }
        },
        "objects" => vec![
            dict! {
                "uid" => uid_b,
                "type" => "alfred.workflow.output.clipboard",
                "config" => dict! {
                    "clipboardtext" => "{query}"
                }
            },
            dict! {
                "uid" => uid_a,
                "type" => "alfred.workflow.input.scriptfilter",
                "config" => dict! {
                    "keyword" => info.keyword,
                    "withspace" => true,
                    // Argument optional
                    "argumenttype" => 1,
                    // Placeholder title
                    "title" => "Search",
                    // "Please wait" subtext
                    "runningsubtext" => "Loading...",
                    // External script
                    "type" => 8,
                    "scriptfile" => info.bin_name,
                    // Terminate previous script
                    "queuemode" => 2,
                    // Always run immediately for first typed character
                    "queuedelayimmediatelyinitially" => true,
                    // Don't set argv when empty
                    "argumenttreatemptyqueryasnil" => true
                }
            }
        ]
    }
}

fn sync_directory() -> Result<PathBuf> {
    let home = home::home_dir().context("failed to get home directory")?;
    let prefs = home.join("Library/Preferences/com.runningwithcrayons.Alfred-Preferences.plist");
    let prefs = plist::Value::from_file(&prefs)?;
    let dir = match prefs
        .into_dictionary()
        .context("expected dictionary")?
        .remove("syncfolder")
    {
        Some(dir) => {
            let dir = PathBuf::from(dir.into_string().context("expected string")?);
            if let Ok(p) = dir.strip_prefix("~") {
                home.join(p)
            } else {
                dir
            }
        }
        None => home.join("Library/Application Support/Alfred"),
    };
    Ok(dir)
}

pub fn workflows_directory() -> Result<PathBuf> {
    Ok(sync_directory()?.join("Alfred.alfredpreferences/workflows"))
}

pub fn package(src_dir: &Path, dst: &Path) -> Result<()> {
    let file = fs::File::create(&dst)?;
    let mut zip = zip::ZipWriter::new(file);

    for entry in src_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap().to_str().unwrap();

        // preserve file permissions
        let mode = path.metadata()?.permissions().mode();
        let options = zip::write::FileOptions::default().unix_permissions(mode);

        if path.is_file() {
            zip.start_file(name, options)?;
            zip.write_all(&fs::read(path)?)?;
        } else {
            zip.add_directory(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
