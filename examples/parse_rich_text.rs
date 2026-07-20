use thar::rich_text::RichText;

fn main() -> std::io::Result<()> {
    for arg in std::env::args().skip(1) {
        match RichText::parse(&arg) {
            Ok(rich_text) => {
                println!("{:#?}", rich_text);
            }
            Err(error) => {
                eprintln!("{:#?}\n{error}", error.location());
                error.print_line(&arg, &mut std::io::stderr())?;
            }
        }
    }
    Ok(())
}
