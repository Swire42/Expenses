mod interface;
mod tags;
mod accounts;
mod transaction;
mod money;
mod yamlrw;

use std::error::Error;

fn app() -> Result<(), Box<dyn Error>> {
    use yamlrw::YamlRW;
    println!("{:?}", tags::Tags::read_yaml("tags.yaml")?);
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
