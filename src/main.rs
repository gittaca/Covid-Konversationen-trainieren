#![allow(unused_must_use)]

use std::fs;

use chrono::Utc;
use clap::Clap;
use comrak::{markdown_to_html, ComrakOptions};
use csv::Writer;
use regex::Regex;
use std::error::Error;

struct AnkiCard {
    front: String,
    back: String,
    card_type: AnkiCardType,
    tags: Vec<String>,
}

#[derive(Debug)]
enum AnkiCardType {
    Basic,
    BasicWithReverse,
    Cloze,
}

const DEFAULT_OUT_FILEPATH: &'static str = "496b076e";

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Niko Linnansalo <niko@linnansalo.com>")]
struct Opts {
    #[clap(short, long, default_value = "anki.md")]
    input_file: String,
    #[clap(short, long, default_value = DEFAULT_OUT_FILEPATH)]
    output_file: String,
    #[clap(short, long)]
    verbose: bool,
}

pub fn main() {
    let opts: Opts = Opts::parse();
    let raw_markdown: String = read_markdown(opts.input_file);
    let anki_cards: Vec<AnkiCard> = make_anki_cards(raw_markdown);
    make_output_csv(&anki_cards, opts.output_file, opts.verbose);
}

fn read_markdown(filepath: String) -> String {
    fs::read_to_string(filepath).expect("Something went wrong reading the file")
}

fn make_anki_cards(raw_markdown: String) -> Vec<AnkiCard> {
    let mut anki_cards: Vec<AnkiCard> = Vec::new();

    let mut temp_front: String = "".to_string();
    let mut temp_back: String = "".to_string();

    for line in raw_markdown.split("\n") {
        // Card front is one line and starts
        // with ##. E.g. ## [Rust, udemy]
        if line.starts_with("## ") {
            if !temp_front.is_empty() {
                anki_cards.push(AnkiCard {
                    front: process_front(&temp_front),
                    back: process_back(&temp_back),
                    card_type: determine_card_type(&temp_front),
                    tags: find_tags(&temp_front, false),
                });
            }

            temp_front = line.to_string();
            temp_back = "".to_string();
        } else if line.starts_with("---") {
            temp_front = temp_front + &temp_back;
            temp_back = "".to_string();

        // All lines between ##'s belong
        // to the back side of a card
        } else {
            temp_back = temp_back + line + "\n";
        }
    }

    // Add last card after exited loop
    if !temp_back.is_empty() {
        anki_cards.push(AnkiCard {
            front: process_front(&temp_front),
            back: process_back(&temp_back),
            card_type: determine_card_type(&temp_front),
            tags: find_tags(&temp_front, false),
        })
    }

    anki_cards
}

fn process_front(front: &String) -> String {
    convert_markdown_to_html(&front[3..].to_string())
}

fn process_back(back: &String) -> String {
    convert_markdown_to_html(back)
}

fn convert_markdown_to_html(input_markdown: &String) -> String {
    let mut html_string: String = markdown_to_html(&input_markdown, &ComrakOptions::default());

    html_string = str::replace(&html_string, "<pre", "<pre align=left ");
    html_string = str::replace(&html_string, "<ul", "<ul align=left ");
    html_string = str::replace(&html_string, "<ol", "<ol align=left ");

    html_string
}

fn determine_card_type(front: &String) -> AnkiCardType {
    for tag in find_tags(front, true) {
        if "REV" == tag {
            return AnkiCardType::BasicWithReverse;
        } else if "CLO" == tag {
            return AnkiCardType::Cloze;
        }
    }

    AnkiCardType::Basic
}

fn find_tags(front: &String, keep_card_type_tags: bool) -> Vec<String> {
    // Treat all terms in first [] as a tag literal
    // E.g.: [Rust, udemy]
    let re = Regex::new(r"\[.*\]").unwrap();

    let matched_string: String = re
        .captures(&process_front(front))
        .unwrap()
        .get(0)
        .map_or("".to_string(), |m| m.as_str().to_string());

    let matched_string = &matched_string[1..matched_string.len() - 1];

    let mut tag_vector: Vec<String> = Vec::new();

    tag_vector.push("ankimd".to_string());

    for tag in matched_string.split(", ") {
        let card_type_tags = vec!["BAS", "REV", "CLO"];

        if card_type_tags.contains(&tag) && !keep_card_type_tags {
            continue;
        }
        tag_vector.push(tag.to_string());
    }

    tag_vector
}

fn make_output_csv(
    anki_cards: &Vec<AnkiCard>,
    output_filepath: String,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    let mut _filepath = output_filepath.clone();

    if _filepath == DEFAULT_OUT_FILEPATH {
        let _outputdir = Utc::now().format("csv_outputs/%Y-%m-%d_%H/").to_string();
        fs::create_dir_all(&_outputdir);
        _filepath = _outputdir + "basic.csv"
    }

    let mut wtr = Writer::from_path(_filepath.clone())?;

    let mut all_tags = Vec::new();

    for card in anki_cards {
        if verbose {
            println!("Front:\n{:?}\n", card.front);
            println!("Back:\n{:?}\n", card.back);
            println!("Tags: {:?}\n", card.tags);
            println!("Type: {:?}", card.card_type);
        }

        all_tags.extend(card.tags.iter().cloned());
        wtr.write_record(&[
            &card.front,
            &card.back,
            &card.tags.join(" "),
            &format!("{:?}", card.card_type),
        ])?;
    }

    wtr.flush()?;

    println!(
        "\nWrote {} cards to filepath {}",
        anki_cards.len(),
        _filepath
    );
    println!("Found {} tags in cards: {:?}", all_tags.len(), all_tags);
    Ok(())
}
