use std::collections::HashMap;
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////
// Definitions
////////////////////////////////////////////////////////////////////////////////

/// Alias for a lean clone-on-write string.
pub type String<'a> = beef::lean::Cow<'a, str>;

/// A keyboard modifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModifierKey {
    /// ⌘
    Command,
    /// ⌥
    Option,
    /// ⌃
    Control,
    /// ⇧
    Shift,
    /// fn
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Default,
    File,
    FileSkipCheck,
}

/// The copied or large type text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text<'a> {
    /// Defines the text the user will get when copying the item (⌘+C).
    copy: Option<String<'a>>,
    /// Defines the text the user will see in large type (⌘+L).
    large_type: Option<String<'a>>,
}

/// The settings for when a modifier key is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierData<'a> {
    /// The subtitle displayed in the result row.
    subtitle: Option<String<'a>>,
    /// The argument which is passed through to the output.
    arg: Option<String<'a>>,
    /// The icon displayed in the result row when the modifier is pressed.
    icon: Option<Icon<'a>>,
    /// Mark whether the item is valid when the modifier is pressed.
    valid: Option<bool>,
}

/// An Alfred script filter item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item<'a> {
    /// The title displayed in the result row.
    title: String<'a>,
    /// The subtitle displayed in the result row.
    subtitle: Option<String<'a>>,
    /// A unique identifier for the item.
    uid: Option<String<'a>>,
    /// The argument which is passed through to the output.
    arg: Option<String<'a>>,
    /// The icon displayed in the result row.
    icon: Option<Icon<'a>>,
    /// Whether this item is valid or not.
    valid: Option<bool>,
    /// Enables you to define what Alfred matches against.
    matches: Option<String<'a>>,
    /// Populates the search field when the user auto-completes the result.
    autocomplete: Option<String<'a>>,
    /// The type of item.
    kind: Kind,
    /// Control how the modifier keys react.
    modifiers: HashMap<ModifierKey, ModifierData<'a>>,
    /// Defines the copied or large type text for this item.
    text: Option<Text<'a>>,
    /// A Quick Look URL which will be shown if the user uses Quick Look (⌘+Y).
    quicklook_url: Option<String<'a>>,
}

/// The output of a workflow (i.e. input for the script filter)
pub struct Output {
    /// Each row item.
    items: Vec<Item>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

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
        pub fn $name<V>(mut self, value: V) -> Self
        where
            V: Into<$ty>,
        {
            self.$name = Some(value.into());
            self
        }
    };
    ($name:ident, $ty:ty) => {
        pub fn $name<V>(mut self, value: V) -> Self
        where
            V: Into<$ty>,
        {
            self.$name = value.into();
            self
        }
    };
}

impl<'a> Item<'a> {
    /// Create a new item.
    ///
    /// # Examples
    ///
    /// ```
    /// use powerpack::Item;
    ///
    /// let item = Item::new("something");
    /// ```
    pub fn new<S>(title: S) -> Self
    where
        S: Into<String<'a>>,
    {
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
    setter! { quicklook_url }
}
