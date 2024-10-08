
use std::io::Write;



pub struct StdoutWriter { }
impl StdoutWriter {}
impl Write for StdoutWriter {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let mut stdout = std::io::stdout();
    for byte in buf {
      match byte {
        b'\n' => { crossterm::queue!(stdout, crossterm::style::Print("\n\r"))?; },
        _ => {
          crossterm::queue!(
            stdout,
            crossterm::style::Print(byte.as_ascii().expect("invalid ascii entered"))
          )?;
        },
      }
    }
    return Ok(buf.len());
  }

  fn flush(&mut self) -> std::io::Result<()> {
    return std::io::stdout().flush();
  }
}



pub fn expect_log_error(error: anyhow::Error) {
  crossterm::queue!(std::io::stdout(),
    crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
    crossterm::style::Print("Err"),
    crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
    crossterm::style::Print(format!(": {}\n\r", error))
  ).expect("unable to log error to stdout");
}


#[inline] pub fn display_command_line(cursor: usize, command: &str) -> anyhow::Result<()> {
  // let run_prompt = crate::runner::generate_run_prompt_string("pwd", &[])?;
  let run_prompt = crate::runner::generate_run_prompt_string()?;
  // let run_prompt = crate::runner::run_command("prompt", &[])
  //   .output().unwrap().stdin.as_slice();
  // let run_prompt = crate::runner::await_command("prompt", &[])?;
  use crossterm::{terminal::{Clear, ClearType}, cursor::MoveToColumn, style::Print};
  crossterm::execute!( std::io::stdout(),
    Clear(ClearType::CurrentLine),
    MoveToColumn(0),
    Print(&run_prompt[0..run_prompt.len()]),
    // Print("> "),
    Print(command),
    MoveToColumn((cursor + run_prompt.len()) as u16),
  )?;

  return Ok(());
}


// any error here should be .expect()'ed because it the message
// may not have been logged properly which might make debugging harder
pub fn log_msg<T: std::fmt::Display>(msg: T) -> anyhow::Result<()> {
  let mut stdout = StdoutWriter{};
  let mut logfile = std::fs::OpenOptions::new()
    .append(true)
    .open(crate::LOGFILE)?;
  let mut message = msg.to_string();
  message.push('\n');

  logfile.write(message.as_bytes())?;
  stdout.write(message.as_bytes())?;

  return Ok(());
}

// any error here should be .expect()'ed because it the message
// may not have been logged properly which might make debugging harder
pub fn log_err<T: std::fmt::Display>(msg: T) -> anyhow::Result<()> {
  let mut stdout = StdoutWriter{};
  let mut logfile = std::fs::OpenOptions::new()
    .append(true)
    .open(crate::LOGFILE)?;

  let mut message = msg.to_string();
  message.push('\n');

  logfile.write(b"Err: ")?;
  logfile.write(message.as_bytes())?;

  use crossterm::style::{Color, SetForegroundColor};
  crossterm::queue!(stdout,
    SetForegroundColor(Color::Red),
    crossterm::style::Print("Err: "),
    SetForegroundColor(Color::Reset),
    crossterm::style::Print(message.as_str()),
  )?;

  
  return Ok(());
}


