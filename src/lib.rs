//! ⚡ Supercharge your Alfred workflows by building them in Rust!
//!
//! # Introduction
//!
//! This crate provides types for developing script filter Alfred workflows in
//! Rust. Additionally, this project includes the `powerpack-cli` crate which
//! contains a command-line tool to help build and install your workflows.
//!
//! Types in this crate closely mirror the script filter JSON format. View the
//! official documentation for that [here][fmt].
//!
//! [fmt]: https://www.alfredapp.com/help/workflows/inputs/script-filter/json/
//!
//! # Examples
//!
//! Each row in an Alfred script filter result is represented by an [`Item`]. A
//! workflow must output a sequence of items to stdout using the [`output()`]
//! function.
//!
//! ```
//! use std::iter;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let item = powerpack::Item::new("Example title")
//!     .subtitle("example subtitle")
//!     .arg("example");
//!
//! powerpack::output(iter::once(item))?;
//! # Ok(())
//! # }
//! ```

pub mod env;

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

////////////////////////////////////////////////////////////////////////////////
// Definitions
////////////////////////////////////////////////////////////////////////////////

/// Alias for a lean clone-on-write string.
pub type String<'a> = beef::lean::Cow<'a, str>;

/// A keyboard modifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ModifierKey {
    /// ⌘
    #[serde(rename = "cmd")]
    Command,
    /// ⌥
    #[serde(rename = "alt")]
    Option,
    /// ⌃
    #[serde(rename = "ctrl")]
    Control,
    /// ⇧
    #[serde(rename = "shift")]
    Shift,
    /// fn
    #[serde(rename = "fn")]
    Function,
}

/// An icon displayed in the result row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Icon<'a> {
    /// Load an image from a path.
    Path(PathBuf),
    /// Extract the icon from a file.
    File(PathBuf),
    /// Uniform Type Identifier (UTI) icon.
    FileType(String<'a>),
}

/// The type of item.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Kind {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "file:skipcheck")]
    FileSkipCheck,
}

/// The copied or large type text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Text<'a> {
    /// Defines the text the user will get when copying the item (⌘+C).
    copy: Option<String<'a>>,

    /// Defines the text the user will see in large type (⌘+L).
    #[serde(rename = "largetype")]
    large_type: Option<String<'a>>,
}

/// The settings for when a modifier key is pressed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
pub struct ModifierData<'a> {
    /// The subtitle displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String<'a>>,

    /// The argument which is passed through to the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<String<'a>>,

    /// The icon displayed in the result row when the modifier is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon<'a>>,

    /// Mark whether the item is valid when the modifier is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,
}

/// An Alfred script filter item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Item<'a> {
    /// The title displayed in the result row.
    title: String<'a>,

    /// The subtitle displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String<'a>>,

    /// A unique identifier for the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String<'a>>,

    /// The argument which is passed through to the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<String<'a>>,

    /// The icon displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon<'a>>,

    /// Whether this item is valid or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,

    /// Enables you to define what Alfred matches against.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    matches: Option<String<'a>>,

    /// Populates the search field when the user auto-completes the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    autocomplete: Option<String<'a>>,

    /// The type of item.
    #[serde(rename = "type", skip_serializing_if = "is_default")]
    kind: Kind,

    /// Control how the modifier keys react.
    #[serde(rename = "mods", skip_serializing_if = "HashMap::is_empty")]
    modifiers: HashMap<ModifierKey, ModifierData<'a>>,

    /// Defines the copied or large type text for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Text<'a>>,

    /// A Quick Look URL which will be shown if the user uses Quick Look (⌘+Y).
    #[serde(rename = "quicklookurl", skip_serializing_if = "Option::is_none")]
    quicklook_url: Option<String<'a>>,
}

/// The output of a workflow (i.e. input for the script filter)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Output<'a> {
    /// Each row item.
    items: Vec<Item<'a>>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Serialize for Icon<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Path(path) => {
                let mut s = serializer.serialize_struct("Icon", 1)?;
                s.serialize_field("path", &path)?;
                s.end()
            }
            Self::File(path) => {
                let mut s = serializer.serialize_struct("Icon", 2)?;
                s.serialize_field("type", "fileicon")?;
                s.serialize_field("path", &path)?;
                s.end()
            }
            Self::FileType(string) => {
                let mut s = serializer.serialize_struct("Icon", 2)?;
                s.serialize_field("type", "filetype")?;
                s.serialize_field("path", &string)?;
                s.end()
            }
        }
    }
}

impl<'a> Icon<'a> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    pub fn from_file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    pub fn from_file_type(path: impl Into<String<'a>>) -> Self {
        Self::FileType(path.into())
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Default
    }
}

macro_rules! setter {
    ($name:ident) => {
        setter! { $name, Option<String<'a>> }
    };
    ($name:ident, Option<$ty:ty>) => {
        pub fn $name(mut self, value: impl Into<$ty>) -> Self {
            self.$name = Some(value.into());
            self
        }
    };
    ($name:ident, $ty:ty) => {
        pub fn $name(mut self, value: impl Into<$ty>) -> Self {
            self.$name = value.into();
            self
        }
    };
}

impl<'a> ModifierData<'a> {
    /// Create a new modifier data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack::ModifierData;
    /// let data = ModifierData::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    setter! { subtitle }
    setter! { arg }
    setter! { icon, Option<Icon<'a>> }
    setter! { valid, Option<bool> }
}

impl<'a> Item<'a> {
    /// Create a new item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack::Item;
    /// let item = Item::new("something");
    /// ```
    pub fn new(title: impl Into<String<'a>>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    setter! { subtitle }
    setter! { uid }
    setter! { arg }
    setter! { icon, Option<Icon<'a>> }
    setter! { valid, Option<bool> }
    setter! { matches }
    setter! { autocomplete }
    setter! { kind, Kind }
    setter! { text, Option<Text<'a>> }
    setter! { quicklook_url }

    pub fn modifier(mut self, key: ModifierKey, data: ModifierData<'a>) -> Self {
        self.modifiers.insert(key, data);
        self
    }
}

impl<'a> Output<'a> {
    pub fn items<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = Item<'a>>,
    {
        self.items = iter.into_iter().collect();
        self
    }

    pub fn write<W: io::Write>(&self, w: W) -> serde_json::Result<()> {
        serde_json::to_writer(w, self)
    }
}

/// Shortcut function to output a list of items to stdout.
pub fn output<'a, I>(items: I) -> serde_json::Result<()>
where
    I: IntoIterator<Item = Item<'a>>,
{
    Output::default().items(items).write(io::stdout())
}
