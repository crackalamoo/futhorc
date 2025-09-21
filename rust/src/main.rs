#[cfg(feature = "zip")]
fn main() -> Result<(), anyhow::Error> {
    use rust_futhorc::futhorc::EnglishToRunes;
    use std::io;

    let dictionary = EnglishToRunes::default();

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        print!("{}", dictionary.translate(line));
    }
}

#[cfg(not(feature = "zip"))]
fn main() {
    panic!("You didn't enable the feature zip.");
}
