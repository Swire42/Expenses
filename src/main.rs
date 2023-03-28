mod interface;
mod tags;
mod accounts;
mod transaction;
mod money;
mod yamlrw;
mod completion;

use std::error::Error;

fn app() -> Result<(), Box<dyn Error>> {
    use yamlrw::YamlRW;
    let mut tags = tags::Tags::read_yaml("tags.yaml")?;
    tags.fix();
    println!("{:?}", tags);
    println!("{:?}", accounts::Accounts::read_yaml("accounts.yaml")?);
    interface::app().unwrap();
    Ok(())
}

fn main() {
    match app() {
        Err(err) => eprintln!("{err}"),
        _ => (),
    }
}
