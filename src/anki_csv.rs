#![allow(unused_must_use)]

mod formatters;
mod io;
mod schema;
mod tags;

pub fn make(input_file: &String, output_file: &String, verbose: bool, uses_date_folder: bool) {
    let raw_markdown: String = io::read_markdown(input_file, verbose);
    let anki_cards: Vec<schema::AnkiCard> = make_anki_cards(&raw_markdown);
    io::make_output_csv(
        &anki_cards,
        output_file.to_string(),
        verbose,
        uses_date_folder,
    );
    io::write_history(raw_markdown);
}

pub fn make_anki_cards(raw_markdown: &str) -> Vec<schema::AnkiCard> {
    let mut anki_cards: Vec<schema::AnkiCard> = Vec::new();

    let mut temp_front: String = "".to_string();
    let mut temp_back: String = "".to_string();

    for line in raw_markdown.split("\n") {
        // Card front is one line and starts
        // with ##. E.g. ## [Rust, udemy]
        if line.starts_with("## ") {
            if !temp_front.is_empty() {
                anki_cards.push(schema::AnkiCard {
                    front: formatters::format_front(&temp_front),
                    back: formatters::format_back(&temp_back),
                    card_type: tags::determine_card_type(&temp_front),
                    tags: tags::find_tags(&temp_front, false),
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
        anki_cards.push(schema::AnkiCard {
            front: formatters::format_front(&temp_front),
            back: formatters::format_back(&temp_back),
            card_type: tags::determine_card_type(&temp_front),
            tags: tags::find_tags(&temp_front, false),
        })
    }

    anki_cards
}
