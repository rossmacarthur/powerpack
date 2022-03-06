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
use std::time::Duration;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[cfg(feature = "detach")]
pub use powerpack_detach as detach;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

////////////////////////////////////////////////////////////////////////////////
// Definitions
////////////////////////////////////////////////////////////////////////////////

/// A keyboard modifier key.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Key {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IconInner {
    /// Load an image from a path.
    Image(PathBuf),
    /// An object whose icon should be shown.
    FileIcon(PathBuf),
    /// Uniform Type Identifier (UTI) icon.
    FileType(String),
}

/// An icon for an [`Item`].
///
/// If not provided the icon will default to the workflow icon.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Icon(IconInner);

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
struct Text {
    /// Defines the text the user will get when copying the item (⌘+C).
    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<String>,

    /// Defines the text the user will see in large type (⌘+L).
    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    large_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
struct Data {
    /// The subtitle displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,

    /// The argument which is passed through to the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<String>,

    /// The icon displayed in the result row when the modifier is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon>,

    /// Mark whether the item is valid when the modifier is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,
}

/// The modifier settings for an [`Item`] when a modifier key is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Modifier {
    /// The modifier key.
    key: Key,

    /// The modifier data.
    data: Data,
}

/// An Alfred script filter item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Item {
    /// The title displayed in the result row.
    title: String,

    /// The subtitle displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,

    /// A unique identifier for the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,

    /// The argument which is passed through to the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<String>,

    /// The icon displayed in the result row.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon>,

    /// Whether this item is valid or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,

    /// Enables you to define what Alfred matches against.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    matches: Option<String>,

    /// Populates the search field when the user auto-completes the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    autocomplete: Option<String>,

    /// The type of item.
    #[serde(rename = "type", skip_serializing_if = "is_default")]
    kind: Kind,

    /// Control how the modifier keys react.
    #[serde(rename = "mods", skip_serializing_if = "HashMap::is_empty")]
    modifiers: HashMap<Key, Data>,

    /// Defines the copied or large type text for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Text>,

    /// A Quick Look URL which will be shown if the user uses Quick Look (⌘+Y).
    #[serde(rename = "quicklookurl", skip_serializing_if = "Option::is_none")]
    quicklook_url: Option<String>,
}

/// The output of a workflow (i.e. input for the script filter)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Output {
    /// The interval in seconds after which to rerun the script filter.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "duration_as_secs"
    )]
    rerun: Option<Duration>,

    /// Each row item.
    items: Vec<Item>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Serialize for Icon {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            IconInner::Image(path) => {
                let mut s = serializer.serialize_struct("Icon", 1)?;
                s.serialize_field("path", &path)?;
                s.end()
            }
            IconInner::FileIcon(path) => {
                let mut s = serializer.serialize_struct("Icon", 2)?;
                s.serialize_field("type", "fileicon")?;
                s.serialize_field("path", &path)?;
                s.end()
            }
            IconInner::FileType(string) => {
                let mut s = serializer.serialize_struct("Icon", 2)?;
                s.serialize_field("type", "filetype")?;
                s.serialize_field("path", &string)?;
                s.end()
            }
        }
    }
}

impl Icon {
    /// Create a new icon using the image at the given path.
    ///
    /// This path can be relative to the workflow directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack::Icon;
    /// let icon = Icon::with_image("./assets/icon.png");
    /// ```
    pub fn with_image(path: impl Into<PathBuf>) -> Self {
        Self(IconInner::Image(path.into()))
    }

    /// Create a new icon based on the file provided.
    ///
    /// This path can be relative to the workflow directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack::Icon;
    /// let icon = Icon::with_file_icon("./assets/example.jpg");
    /// ```
    ///
    /// The above code would use the following icon:
    ///
    /// <img src="https://user-images.githubusercontent.com/17109887/118356177-4695fa80-b574-11eb-8908-c0ccd5f6d23c.png" height="50"/>
    ///
    /// You could combine with "/Applications/Safari.app" to show Safari's icon:
    ///
    /// ```
    /// # use powerpack::Icon;
    /// let icon = Icon::with_file_icon("/Applications/Safari.app");
    /// ```
    pub fn with_file_icon(path: impl Into<PathBuf>) -> Self {
        Self(IconInner::FileIcon(path.into()))
    }

    /// Create a new icon using an Apple [Uniform Type Identifier (UTI)][uti].
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack::Icon;
    /// let icon = Icon::with_type("public.jpeg");
    /// ```
    ///
    /// The above code would use the following icon:
    ///
    /// <img src="https://user-images.githubusercontent.com/17109887/118356177-4695fa80-b574-11eb-8908-c0ccd5f6d23c.png" height="50"/>
    ///
    /// [uti]: https://en.wikipedia.org/wiki/Uniform_Type_Identifier
    pub fn with_type(uti: impl Into<String>) -> Self {
        Self(IconInner::FileType(uti.into()))
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Default
    }
}

impl Modifier {
    /// Create a new modifier.
    #[must_use]
    pub fn new(key: Key) -> Self {
        Self {
            key,
            data: Data::default(),
        }
    }

    /// The subtitle for when this modifier is activated.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.data.subtitle = Some(subtitle.into());
        self
    }

    /// The arg for when this modifier is activated.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.data.arg = Some(arg.into());
        self
    }

    /// The icon for when this modifier is activated.
    #[must_use]
    pub fn icon(mut self, arg: impl Into<Icon>) -> Self {
        self.data.icon = Some(arg.into());
        self
    }

    /// Whether this item is valid when the modifier is activated.
    #[must_use]
    pub fn valid(mut self, valid: impl Into<bool>) -> Self {
        self.data.valid = Some(valid.into());
        self
    }
}

impl Item {
    /// Create a new item with the provided title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Set the subtitle for this item.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the UID for this item.
    ///
    /// This is a unique identifier for the item which allows help Alfred to
    /// learn about this item for subsequent sorting and ordering of the user's
    /// actioned results.
    ///
    /// It is important that you use the same UID throughout subsequent
    /// executions of your script to take advantage of Alfred's knowledge and
    /// sorting. If you would like Alfred to always show the results in the
    /// order you return them from your script, exclude the UID field.
    #[must_use]
    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    /// Set the argument which is passed through the workflow to the connected
    /// output action.
    ///
    /// While this attribute is optional, it's highly recommended that you
    /// populate this as it's the string which is passed to your connected
    /// output actions. If excluded, you won't know which result item the user
    /// has selected.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self
    }

    /// Set the icon displayed in the result row.
    ///
    /// Workflows are run from their workflow folder, so you can reference icons
    /// stored in your workflow relatively.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether this item is valid or not.
    ///
    /// If an item is valid then Alfred will action this item when the user
    /// presses return. If the item is not valid, Alfred will do nothing. This
    /// allows you to intelligently prevent Alfred from actioning a result based
    /// on the current query passed into your script.
    ///
    /// If you exclude the valid attribute, Alfred assumes that your item is
    /// valid.
    #[must_use]
    pub fn valid(mut self, valid: impl Into<bool>) -> Self {
        self.valid = Some(valid.into());
        self
    }

    /// Set the text that Alfred will match against.
    ///
    /// This field enables you to define what Alfred matches against when the
    /// workflow is set to "Alfred Filters Results". If match is present, it
    /// fully replaces matching on the title property.
    ///
    /// Note that the match field is always treated as case insensitive, and
    /// intelligently treated as diacritic insensitive. If the search query
    /// contains a diacritic, the match becomes diacritic sensitive.
    #[must_use]
    pub fn matches(mut self, matches: impl Into<String>) -> Self {
        self.matches = Some(matches.into());
        self
    }

    /// Set the autocomplete value for this item.
    ///
    /// An optional but recommended string you can provide which is populated
    /// into Alfred's search field if the user auto-complete's the selected
    /// result (⇥ by default).
    #[must_use]
    pub fn autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.autocomplete = Some(autocomplete.into());
        self
    }

    /// Set the type of item.
    #[must_use]
    pub fn kind(mut self, kind: impl Into<Kind>) -> Self {
        self.kind = kind.into();
        self
    }

    /// Set the text the user will get when copying the selected result row with
    /// ⌘C or displaying large type with ⌘L.
    ///
    /// If these are not defined, you will inherit Alfred's standard behaviour
    /// where the arg is copied to the Clipboard or used for Large Type.
    #[must_use]
    pub fn copy_text(mut self, copy: impl Into<String>) -> Self {
        self.text.get_or_insert_with(Text::default).copy = Some(copy.into());
        self
    }

    /// Set the text the user will get when displaying large type with ⌘L.
    ///
    /// If this is not defined, you will inherit Alfred's standard behaviour
    /// where the arg is used for Large Type.
    #[must_use]
    pub fn large_type_text(mut self, large_type: impl Into<String>) -> Self {
        self.text.get_or_insert_with(Text::default).large_type = Some(large_type.into());
        self
    }

    /// Set the Quick Look URL for the item.
    ///
    /// This will be visible if the user uses the Quick Look feature within
    /// Alfred (tapping shift, or ⌘Y). This field will also accept a file path,
    /// both absolute and relative to home using ~/.
    ///
    /// If absent, Alfred will attempt to use the arg as the quicklook URL.
    #[must_use]
    pub fn quicklook_url(mut self, quicklook_url: impl Into<String>) -> Self {
        self.quicklook_url = Some(quicklook_url.into());
        self
    }

    /// Add a modifier key configuration.
    ///
    /// This gives you control over how the modifier keys react. For example you
    /// can define the valid attribute to mark if the result is valid based on
    /// the modifier selection and set a different arg to be passed out if
    /// actioned with the modifier.
    #[must_use]
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        let Modifier { key, data } = modifier;
        self.modifiers.insert(key, data);
        self
    }
}

fn duration_as_secs<S>(duration: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match duration {
        Some(d) => s.serialize_f32(d.as_secs_f32()),
        None => unreachable!(),
    }
}

impl Output {
    /// Create a new output.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the rerun value.
    ///
    /// Scripts can be set to re-run automatically after an interval with a
    /// value of 0.1 to 5.0 seconds. The script will only be re-run if the
    /// script filter is still active and the user hasn't changed the state of
    /// the filter by typing and triggering a re-run.
    pub fn rerun(&mut self, duration: Duration) -> &mut Self {
        self.rerun = Some(duration);
        self
    }

    /// Extend the list of items to output.
    pub fn items<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Item>,
    {
        self.items.extend(iter);
        self
    }

    /// Output this script filter to the given writer.
    pub fn write<W: io::Write>(&self, w: W) -> serde_json::Result<()> {
        serde_json::to_writer(w, self)
    }
}

/// Shortcut function to output a list of items to stdout.
pub fn output<I>(items: I) -> serde_json::Result<()>
where
    I: IntoIterator<Item = Item>,
{
    Output::new().items(items).write(io::stdout())
}
