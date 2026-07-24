
use clap::Parser;
use thar::{rich_text::RichText, wrap::wrap};

#[derive(Parser)]
struct Args {
    #[clap(short = 'w', long)]
    width: usize,
    #[clap(short = 'r', long)]
    rich_text: bool,
    text: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.rich_text {
        let rich_text = match RichText::parse(&args.text) {
            Ok(rich_text) => rich_text,
            Err(error) => {
                eprintln!("{:#?}\n{error}", error.location());
                error.print_line(&args.text, &mut std::io::stderr())?;
                std::process::exit(1);
            }
        };
        let rich_text = rich_text.wrap(args.width);

        rich_text.print(&mut std::io::stdout())?;
    } else {
        for line in wrap(&args.text, args.width) {
            println!("{line}");
        }
    }

    Ok(())
}
