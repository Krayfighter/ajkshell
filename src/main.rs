#![feature(ascii_char)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
// #![feature(new_range_api)]

#[macro_use] extern crate anyhow;

use std::io::{Read, Write};

mod utils;
mod runner;
mod interface;
mod parser;


pub const LOGFILE: &'static str = "ajkshell_log.txt";


trait ReprAsOsStrTrait: ReprAs<Box<[u8]>> {}
type ReprAsOsStr = std::rc::Rc<dyn ReprAsOsStrTrait>;

trait ReprAs<T> {
  fn repr_as(&self) -> T;
}


pub trait SliceTreeSource: AsRef<[u8]> {}
impl<T: AsRef<[u8]>> SliceTreeSource for T {}

#[derive(Clone)]
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
  pub fn subslice(&self, range: std::ops::Range<usize>) -> Self {
    if (range.start < self.range.start) |
      (range.end > self.range.end) {
    // { panic!("out of range"); }
      interface::log_err("(invalid state) subslice out of range").unwrap();
      panic!("(invalid state)");
    }
    return Self { source: self.source.clone(), range }
  }
  pub fn as_slice<'a>(&'a self) -> &'a[u8] {
    return &self.source.as_ref().as_ref()[self.range.clone()];
  }
}

impl<T: SliceTreeSource> std::fmt::Debug for SliceTree<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f.write_str(&format!("[{}..{}] ({})",
      self.range.start,
      self.range.end,
      utils::as_str(self.as_slice())
    ));
  }
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
  // clear the log file
  std::fs::write(LOGFILE, "")
    .expect("failed to create or write to log file");
  
  // let mut stdout = std::io::stdout();
  let mut writer = interface::StdoutWriter {};

  let path_var = format!(
    "{}{}:{}",
    std::env::current_dir().unwrap().to_str().unwrap(),
    "/build",
    std::env::var("PATH").unwrap(),
  );
  std::env::set_var("PATH", path_var);

  crossterm::terminal::enable_raw_mode()
    .expect("unable to enable raw mode");
  defer!(DisableRawMode, { crossterm::terminal::disable_raw_mode().unwrap(); });
  let _defer_disable_raw_mode = DisableRawMode{};

  let mut command_buffer = String::new();

  let mut history: Vec<String> = vec!();

  'main: loop {

    let mut cursor: usize = 0;

    command_buffer.clear();
    let mut history_index: Option<usize> = Some(0);
    'buffer: loop {
      interface::display_command_line(cursor, &command_buffer).expect("unable to print command line");
      use crossterm::event::Event as CE;
      use crossterm::event::KeyEventKind as CKE;
      use crossterm::event::KeyCode as CKC;
      match crossterm::event::read() {
        Ok(CE::Key(event)) => { match event.kind {
          CKE::Press => match event.code {
            CKC::Char(chr) => {
              // check for control codes
              match event.modifiers {
                crossterm::event::KeyModifiers::CONTROL => {
                  match chr {
                    'd' => {
                      if command_buffer.len() == 0 { break 'main; }
                    },
                    'c' => {
                      writer.write(b"^C\n\r").unwrap();
                      command_buffer.clear();
                      cursor = 0;
                      continue 'buffer;
                    }
                    _ => continue 'buffer,
                  }
                },
                _ => {},
              }
              // otherwise add to command buffer
              command_buffer.push(chr);
              cursor += 1;
            },
            CKC::Backspace => {
              assert!(!(cursor > command_buffer.len()));
              if cursor == 0 { continue 'buffer }
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
            CKC::Up => {
              if history.len() == 0 { continue 'buffer }
              if let None = history_index { history_index = Some(0); }
              let index = history_index.unwrap();
              if index+1 < history.len() { history_index = Some(index+1); }
              command_buffer = history.iter().rev()
                .nth(index)
                .unwrap().clone();
              cursor = command_buffer.len();
            },
            CKC::Down => {
              if let Some(index) = history_index {
                if index == 0 {
                  history_index = None;
                  command_buffer.clear();
                } else {
                  history_index = Some(index-1);
                  command_buffer = history.iter().rev()
                    .nth(history_index.unwrap())
                    .unwrap()
                    .clone();
                }
              }
              cursor = command_buffer.len();
            },
            CKC::Enter => break 'buffer,
            CKC::Esc => break 'main,
            _ => continue 'buffer,
          },
          _ => continue 'buffer,
        }},
        _ => continue 'buffer
      }
    }

    // // TODO this could be updated
    // if command_buffer.len() == 0 {
    //   stdout.write(b"\n\r").unwrap();
    //   stdout.flush().unwrap();
    //   continue;
    // }
    writer.write(b"\n\r").unwrap();
    if command_buffer.len() == 0 { continue 'main; }

    history.push(command_buffer.clone());

    // TODO this is temporary for debug
    if command_buffer.starts_with("exit") { break 'main; }

    let tokens = match parser::lex(&command_buffer) {
      Ok(tokens) => tokens,
      Err(e) => panic!("Err while lexing input: {}", e)
    };

    // each command segment is separated by a pipe
    let command_segments = parser::parse(&tokens).expect("Failed to parse tokens");

    let mut prev_stdout: Option<std::process::ChildStdout> = None;
    let mut prev_stderr: Option<std::process::ChildStderr> = None;
    let mut children: Vec<std::process::Child> = vec!();
    for segment in command_segments.into_iter() {
      let mut command = segment.build(prev_stdout.take().map(|stdout| stdout.into()));
      let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
          interface::log_msg(format!("Failed to run command: {}", e)).unwrap();
          // interface::log_err("failed to run command").unwrap();
          continue 'main;
        }
      };
      prev_stdout = child.stdout.take();
      prev_stderr = child.stderr.take();

      children.push(child);
    }

    for child in children.iter_mut() {
      child.wait().expect("Failed to wait for child process to complete");
    }

    let mut buffer = String::new();

    if let Some(mut stdout) = prev_stdout {
      if let Err(e) = stdout.read_to_string(&mut buffer) {
        interface::log_err(e).unwrap();
      }
    }
    if let Err(e) = writer.write(buffer.as_bytes()) { interface::log_err(e).unwrap(); }

    buffer.clear();
    if let Some(mut stderr) = prev_stderr {
      if let Err(e) = stderr.read_to_string(&mut buffer) {
        interface::log_err(e).unwrap();
      }
    }

    if let Err(e) = writer.write(buffer.as_bytes()) {
      interface::log_err(e).unwrap();
    }

    crossterm::execute!(writer,
      crossterm::style::ResetColor
    ).unwrap();
  }

  crossterm::execute!(
    writer,
    crossterm::style::Print("\n\r"),
    crossterm::style::ResetColor,
  ).unwrap();

  let _ = _defer_disable_raw_mode;
}


