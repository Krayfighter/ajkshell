#![feature(ascii_char)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
// #![feature(new_range_api)]

#[macro_use] extern crate anyhow;

use std::io::Write;

mod utils;
mod runner;
mod builtins;
mod interface;
mod parser;



trait ReprAsOsStrTrait: ReprAs<Box<[u8]>> {}
type ReprAsOsStr = std::rc::Rc<dyn ReprAsOsStrTrait>;

trait ReprAs<T> {
  fn repr_as(&self) -> T;
}


trait SliceTreeSource: AsRef<[u8]> {}
impl<T: AsRef<[u8]>> SliceTreeSource for T {}

#[derive(Debug, Clone)]
pub struct SliceTree<T: SliceTreeSource> {
  source: std::rc::Rc<T>,
  // start inclusive, end exclusive
  range: std::ops::Range<usize>
  // start: usize, end: usize,
}
// this is so that std::rc::Rc::from(SliceTree<_>) works
impl<T: SliceTreeSource> ReprAsOsStrTrait for SliceTree<T> {}
impl<T: SliceTreeSource> AsRef<[u8]> for SliceTree<T> {
  fn as_ref(&self) -> &[u8] { &self.source.as_ref().as_ref()[self.range.clone()] }
}
impl<T: SliceTreeSource> ReprAs<Box<[u8]>> for SliceTree<T> {
  fn repr_as(&self) -> Box<[u8]> { Box::from(self.as_ref()) }
}
impl<T: SliceTreeSource> SliceTree<T> {
  // consumes a source object and contain it
  pub fn consume(item: T) -> Self {
    return Self {
      range: 0..item.as_ref().len(),
      source: std::rc::Rc::from(item),
    }
  }

  // TODO encode this into the type system maybe
  // currently panics if any part of the
  // range argument is greater than the source
  // object's range
  pub fn subslice(&self, range: std::ops::Range<usize>) -> Self {
    if (range.start < self.range.start) |
      (range.end > self.range.end)
    { panic!("out of range"); }
    return Self { source: self.source.clone(), range }
  }
  pub fn as_slice<'a>(&'a self) -> &'a[u8] {
    return &self.source.as_ref().as_ref()[self.range.clone()];
  }
  // pub fn len()
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

  let path_var = format!(
    "{}{}:{}",
    std::env::current_dir().unwrap().to_str().unwrap(),
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

    let tokens = match parser::lex(&command_buffer) {
      Ok(tokens) => tokens,
      Err(e) => panic!("Err while lexing input: {}", e)
    };

    let command_segments = parser::parse(&tokens).expect("Failed to parse tokens");

    // if command_segments.len() = 1 {}
    // else 
    if command_segments.len() > 1 {
      let mut piped_output: Option<std::process::ChildStdout> = None;
      for cmd in command_segments.iter().rev() {
        let mut command = cmd.build();
        if let Some(output) = piped_output {
          command.run().unwrap().stdin(std::process::Stdio::from(output));
        }else { }
        // piped_output = Some(command.run().unwrap().stdout);
        // piped_output = command.spawn().unwrap().stdout;
      }
    }

    // let base_command = &command_segments[0];

    let mut command: std::process::Command = match tokens[0] {
    // let mut command = match command_segments[0] {
      parser::Token::Command(string) => match string {
        "cd" => {
          if let Err(e) = builtins::change_directory(
            tokens.as_slice().into_iter()
              .map(|expr| expr.as_str())
              .collect::<Vec<&str>>()
              .as_slice()
          ) { interface::expect_log_error(e); }
          stdout.write(b"\r\n").unwrap();
          continue;
        },
        "exit" => {
          if let Err(e) = builtins::exit(&tokens_as_slices(&tokens)) {
            interface::expect_log_error(e);
          }else { break; }
          panic!("Impossible condition");
        }
        command => {
          runner::build_command(command,
            &tokens[1..tokens.len()].into_iter()
              .map(|token| token.as_str())
              .collect::<Vec<&str>>()
          )
        }
      }
      // this means that the command start with an argument (invalid state)
      parser::Token::Argument(_) => panic!("can not happend"),
      parser::Token::Pipe => todo!(),
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


