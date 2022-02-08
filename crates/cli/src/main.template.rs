use std::env;
use std::error::Error;
use std::iter;

fn main() -> Result<(), Box<dyn Error>> {
    // Alfred passes in a single argument for the user query.
    let query = env::args().nth(1);

    // Create an item to show in the Alfred drop down.
    let item = powerpack::Item::new("Hello world!")
        .subtitle(format!("Your query was '{:?}'", query))
        .icon(powerpack::Icon::with_type("public.script"));

    // Output the item to Alfred!
    powerpack::output(iter::once(item))?;

    Ok(())
}
