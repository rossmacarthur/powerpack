use indexmap::indexmap;

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
