#![feature(ascii_char)]
#![feature(unboxed_closures)]

#[macro_use] extern crate anyhow;

use std::io::Write;

mod runner;
mod builtins;
mod interface;


#[derive(Debug)]
enum Expr {
  Command(String),
  Argument(String),
  // SubExpr(Option<Vec<Expr>>),
}
impl Expr {
  // pub fn as_os_str(&self) -> std::ffi::OsString {
  //   match self {
  //     Self::Command(string) | Self::Argument(string) => {
  //       return string.into()
  //     }
  //   }
  // }
  pub fn as_str<'a>(&'a self) -> &'a str {
    match self {
      Self::Command(string) | Self::Argument(string) => {
        return &string
      }
    }
  }
}

// fn begin_next_token(
//   current_token_start: usize,
//   index: usize,
// ) -> Option<(usize, usize)> {
//   if current_token_start == index { return None; }
//   if let Some(index) = index.checked_sub(1) {
//     return Some((current_token_start, index));
//   }
//   else { return None; }
// }


#[derive(PartialEq, Eq)]
enum TokenState {
  None,
  InsideToken,
  InsideQuote,
}
impl TokenState {
  pub fn is_isnide_token(&self) -> bool { return *self == Self::InsideToken }
  // pub fn is_none(&self) -> bool { return *self == Self::None }
  pub fn is_inside_quote(&self) -> bool { return *self == Self::InsideQuote }
  // pub fn is_inside_any(&self) -> bool { self.is_isnide_token() | self.is_inside_quote() }
}


fn tokenize(source: &str) -> Vec<Expr> {
  let mut slices: Vec<&str> = Vec::new();
  let mut state = TokenState::None;
  let mut current_slice_start: usize = 0;

  for (index, chr) in source.chars().enumerate() {
    match chr {
      ' ' | '\n' => {
        if !state.is_isnide_token() { continue }

        slices.push(&source[current_slice_start..index]);
        state = TokenState::None;
      },
      '\"' => {
        if state.is_inside_quote() {
          if current_slice_start == index + 1 { slices.push(""); }
          else { slices.push(&source[current_slice_start+1..index]); }
          state = TokenState::None;
          continue
        }
        else {
          if state.is_isnide_token() { panic!("invalid quote placements"); }
          current_slice_start = index;
          state = TokenState::InsideQuote;
          continue
        }
      },
      _ => {
        if state.is_inside_quote() { continue }
        if !state.is_isnide_token() { current_slice_start = index; }
        state = TokenState::InsideToken;
      },
    }
  }

  if current_slice_start != source.len() { slices.push(&source[current_slice_start..source.len()]); }

  let mut tokens: Vec<Expr> = Vec::new();

  let mut command = true;

  for slice in slices.into_iter() {
    if command { tokens.push(Expr::Command(String::from(slice))); command = false; }
    else { tokens.push(Expr::Argument(String::from(slice))); }
  }

  return tokens;
}


macro_rules! defer {
  ($name: ident, $code: block) => {
    struct $name {}
    impl Drop for $name {
      fn drop(&mut self) $code
    }
  };
  ($name: ident, $members: block $code: block) => {
    struct $name {$members}
    impl Drop for $name {
      fn drop(&mut self) $code
    }
  }
}


fn main() {
  let mut stdout = std::io::stdout();
  // let mut path_var = std::env::var("PATH").unwrap();
  // path_var.push(':');
  // path_var.push_str(std::env::current_dir().unwrap().as_os_str().to_str().unwrap());
  // path_var.push_str("/target/debug");
  // let path_postfix = match cfg!(debug) {
  //   false => "/target/debug",
  //   true => "/target/release",
  // }
  let path_var = format!(
    "{}{}:{}",
    std::env::current_dir().unwrap().to_str().unwrap(),
    // "/target/debug",
    // path_postfix,
    "/build",
    std::env::var("PATH").unwrap(),
  );
  std::env::set_var("PATH", path_var);
  println!("PATH: {}", std::env::var("PATH").unwrap());

  crossterm::terminal::enable_raw_mode()
    .expect("unable to enable raw mode");
  defer!(DisableRawMode, { crossterm::terminal::disable_raw_mode().unwrap(); });
  let _defer_disable_raw_mode = DisableRawMode{};

  let mut command_buffer = String::new();

  loop {

    let mut cursor: usize = 0;

    command_buffer.clear();
    loop {
      interface::display_command_line(cursor, &command_buffer).expect("unable to print command line");
      use crossterm::event::Event as CE;
      use crossterm::event::KeyEventKind as CKE;
      use crossterm::event::KeyCode as CKC;
      match crossterm::event::read() {
        Ok(CE::Key(event)) => { match event.kind {
          CKE::Press => match event.code {
            CKC::Char(chr) => {
              command_buffer.push(chr);
              cursor += 1;
            },
            CKC::Backspace => {
              assert!(!(cursor > command_buffer.len()));
              if cursor == 0 { continue }
              if cursor == command_buffer.len() { command_buffer.pop(); }
              else { command_buffer.remove(cursor); }
              cursor -=1;
            }
            CKC::Left => {
              if cursor == 0 { cursor = command_buffer.len(); }
              else { cursor -= 1; }
            },
            CKC::Right => {
              if cursor == command_buffer.len() { cursor = 0; }
              else { cursor += 1; }
            },
            CKC::Up => { todo!() },
            CKC::Down => { todo!() },
            CKC::Enter => break,
            CKC::Esc => return,
            _ => continue,
          },
          _ => continue,
        }},
        _ => continue
      }
    }

    if command_buffer.len() == 0 {
      stdout.write(b"\n\r").unwrap();
      stdout.flush().unwrap();
      continue;
    }

    let tokens = tokenize(&command_buffer);

    if tokens.len() == 1 && tokens[0].as_str() == "exit" { break; }

    let mut command: std::process::Command = match &tokens[0] {
      Expr::Command(string) => match string.as_str() {
        "cd" => {
          if let Err(e) = builtins::change_directory(
            tokens.as_slice().into_iter()
              .map(|expr| expr.as_str())
              .collect::<Vec<&str>>()
              .as_slice()
          ) { interface::expect_log_error(e); }
          stdout.write(b"\r\n").unwrap();
          continue;
        }
        command => {
          runner::run_command(command,
            &tokens[1..tokens.len()].into_iter()
              .map(|token| token.as_str())
              .collect::<Vec<&str>>()
          )
        }
      }
      // this means that the command start with an argument (invalid state)
      Expr::Argument(_) => panic!("can not happend"),
    };

    match command.output() {
      Ok(output) => {
        let mut writer = interface::StdoutWriter{};
        writer.write(&[b'\n']).expect("unable to queue in stdout writer");
        writer.write(output.stdout.as_slice()).expect("unable to queue stdout");
        writer.flush().expect("unable to flush writer to stdout");

        if output.stderr.as_slice().len() != 0 {
          writer.write(output.stderr.as_slice()).expect("unable to log stderr to program stdout");
          writer.flush().expect("failed to flush writer to stdout");
        }
      },
      Err(e) => {
        use crossterm::style::{Print, SetForegroundColor, Color};
        crossterm::execute!(
          stdout,
          Print("\n\r"), SetForegroundColor(Color::Red), Print("Err"),
          SetForegroundColor(Color::Reset),
          Print(format!(": {} (missing executable)\n\rTokens: {:?}", e, tokens)),
          crossterm::style::Print("\n\r"),
        ).expect("unable to log error to stdout");
      },
    }
  }

  crossterm::execute!(
    stdout,
    crossterm::style::Print("\n\r"),
  ).unwrap();

  let _ = _defer_disable_raw_mode;
}


