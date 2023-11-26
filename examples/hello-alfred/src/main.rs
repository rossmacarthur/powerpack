use std::env;
use std::error::Error;
use std::io;
use std::time::Duration;

use powerpack::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Alfred passes in a single argument for the user query.
    let query = env::args().nth(1);

    // Create an item to show in the Alfred drop down.
    let item = Item::new("Hello World!")
        .subtitle(format!("Your query was '{query:?}'"))
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
        .quicklook_url("https://example.com");

    // Output the item to Alfred!
    Output::new()
        .rerun(Duration::from_secs(1))
        .skip_knowledge(true)
        .items([item])
        .write(io::BufWriter::new(io::stdout()))?;

    Ok(())
}
