use std::{error::Error, fs};

use clap::Parser;
use dialoguer::{
    theme::{ColorfulTheme},
    Completion, Input,
};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Add an entry
    Add {
        /// Entry amount
        amount: f64,
        /// Entry description
        description: String,
    },
    /// List all entries
    List,
    Settle,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    amount: f64,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct NewEntry {
    payer: String,
    payees: String,
    description: String,
    amount: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    new_add()?;
    new_list()?;
    Ok(())
}

struct SimpleCompletion {
    options: Vec<String>,
}

impl Completion for SimpleCompletion {
    fn get(&self, input: &str) -> Option<String> {
        self.options
            .iter()
            .find(|option| option.starts_with(input))
            .map(String::to_owned)
    }
}

fn new_add() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    fs::DirBuilder::new()
        .recursive(true)
        .create(format!("{home}/.config"))?;

    let file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{home}/.config/new_entries.txt"))?;
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let descriptions = rdr
        .into_deserialize::<NewEntry>()
        .filter_map(Result::ok)
        .map(|entry| entry.description)
        .collect::<Vec<_>>();
    let description_completion = SimpleCompletion {
        options: descriptions,
    };

    let file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{home}/.config/new_entries.txt"))?;
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let people = rdr
        .into_deserialize::<NewEntry>()
        .filter_map(Result::ok)
        .flat_map(|entry| {
            entry
                .payees
                .split(' ')
                .map(str::to_owned)
                .chain([entry.payer])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    println!("{people:?}");
    let people_completion = SimpleCompletion { options: people };

    let payer = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Who paid?")
        .completion_with(&people_completion)
        .interact_text()?;

    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("For what?")
        .completion_with(&description_completion)
        .interact_text()?;

    let amount: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("How much?")
        .interact_text()?;
    let amount = amount.parse::<f64>()?;

    let payees: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("For whom?")
        .interact_text()?;
    let payees = payees.split_whitespace().collect::<Vec<_>>().join(" ");

    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{home}/.config/new_entries.txt"))?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(&file);
    wrt.serialize(NewEntry {
        payer,
        payees,
        description,
        amount,
    })?;

    Ok(())
}

fn new_list() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{home}/.config/new_entries.txt"))?;

    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    for entry in rdr.into_deserialize::<NewEntry>() {
        let NewEntry {
            payer,
            payees,
            description,
            amount,
        } = entry?;
        println!("{description}");
        println!("<- {payer}\t{amount:.2}");
        for payee in payees.split(' ') {
            println!("-> {payee}\t??.??");
        }
    }

    Ok(())
}

fn add(amount: f64, description: String) -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    fs::DirBuilder::new()
        .recursive(true)
        .create(format!("{home}/.config"))?;
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{home}/.config/entries.txt"))?;

    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    wrt.serialize(Entry {
        amount,
        description,
    })?;

    Ok(())
}

fn list() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{home}/.config/entries.txt"))?;

    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    for entry in rdr.into_deserialize::<Entry>() {
        let Entry {
            amount,
            description,
        } = entry?;
        println!("{amount}\t{description}");
    }

    Ok(())
}

fn sum() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{home}/.config/entries.txt"))?;

    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let sum: f64 = rdr
        .into_deserialize::<Entry>()
        .try_fold(0., |sum, entry| entry.map(|entry| entry.amount + sum))?;
    println!("{sum}");

    Ok(())
}
