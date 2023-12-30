use std::time::Duration;

use powerpack::{value, Icon, Item, Key, Kind, Modifier, Output};

#[test]
fn smoke() {
    let item = Item::new("Desktop")
        .uid("desktop")
        .kind(Kind::File)
        .subtitle("~/Desktop")
        .arg("~/Desktop")
        .autocomplete("~/Desktop")
        .icon(Icon::with_file_icon("~/Desktop"));

    let mut output = Output::new();
    output.items([item]);
    goldie::assert_json!(output);
}

#[test]
fn all() {
    let item = Item::new("Hello world!")
        .subtitle("This is a subtitle")
        .uid("unique identifier")
        .arg("/path/to/file.jpg")
        .icon(Icon::with_type("public.jpeg"))
        .valid(true)
        .matches("use this to filter")
        .autocomplete("to this")
        .kind(Kind::FileSkipCheck)
        .copy_text("this text will be copied with ⌘C")
        .large_type_text("this text will be displayed with ⌘L")
        .modifier(Modifier::new(Key::Command).subtitle("⌘ changes the subtitle"))
        .modifier(Modifier::new(Key::Option).arg("/path/to/modified.jpg"))
        .modifier(Modifier::new(Key::Control).icon(Icon::with_image("/path/to/file.png")))
        .modifier(Modifier::new(Key::Shift).valid(false))
        .quicklook_url("https://example.com")
        .action(value!({
            "text": ["one", "two", "three"],
            "url": "https://www.alfredapp.com",
            "file": "~/Desktop",
            "auto": "~/Pictures"
        }));

    let mut output = Output::new();
    output.rerun(Duration::from_millis(500)).items([item]);
    goldie::assert_json!(output);
}
