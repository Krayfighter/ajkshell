


fn main() -> anyhow::Result<()> {

  let dir = std::env::current_dir()?;
  let home = std::env::var("HOME");


  let prompt_string: String = match home {
    Ok(home_dir) => {
      if dir.starts_with(&home_dir) {
        let mut string = String::from("~");
        let dir_str = dir.to_str().unwrap();
        string.push_str(
          &dir_str[home_dir.len()..dir_str.len()]
        );
        string
      }
      else {
        dir.to_str().unwrap().to_owned()
      }
    },
    Err(_) => {
      dir.to_str().unwrap().to_owned()
    }
  };

  use crossterm::style::Print;

  crossterm::execute!(std::io::stdout(),
    Print(prompt_string),
    Print("> "),
  )?;

  return Ok(())

}

