
#[macro_use] extern crate anyhow;

use std::{io::Write, os::unix::{ffi::OsStrExt, fs::FileTypeExt}, path::PathBuf};


use crossterm::style::{Color, Print};

fn queue_color(color: Color, front: bool) -> anyhow::Result<()> {
  if front {
    crossterm::queue!(
      std::io::stdout(),
      crossterm::style::SetForegroundColor(color),
    )?;
  }else {
    crossterm::queue!(
      std::io::stdout(),
      crossterm::style::SetBackgroundColor(color)
    )?;
  }
  return Ok(());
}


struct ActiveArgs {
  // long_list: bool,
  show_hidden_files: bool,
  show_permissions: bool,
}
impl Default for ActiveArgs {
  fn default() -> Self { return Self {
    // long_list: false,
    show_hidden_files: false,
    show_permissions: false,
  }}
}


fn main() -> anyhow::Result<()> {

  let args = std::env::args();

  let mut arg_state = ActiveArgs::default();
  let mut list_dirs: Vec<String> = Vec::new();
  if args.len() == 1 {
    list_dirs.push(std::env::current_dir()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned()
    );
  } // no args
  else { for arg in args {
    if arg.starts_with('-') { match arg.as_str() {
      "-a" => arg_state.show_hidden_files = true,
      "-l" => arg_state.show_permissions = true,
      _ => bail!("unkown argument: {}", arg)
    } }
    else { list_dirs.push(arg); }
  }}

  if arg_state.show_permissions { todo!(); }

  let entries = std::fs::read_dir(
    std::env::current_dir()?
  )?
    .into_iter()
    .map(|dir_entry| dir_entry.unwrap() );
    // .collect::<Vec<PathBuf>>();

  let mut stdout = std::io::stdout();
  for entry in entries {
    queue_color(Color::Reset, true)?;
    queue_color(Color::Reset, false)?;

    let filetype = entry.file_type()?;

    if filetype.is_dir() { queue_color(Color::Green, true)?; }
    // change color for hidden file
    if entry.file_name().as_bytes()[0] == b'.' {
      if arg_state.show_hidden_files {
        queue_color(Color::DarkBlue, false)?;
      }else { continue }
    }
    if filetype.is_symlink() { queue_color(Color::Yellow, false)?; }

    crossterm::queue!(stdout, Print(entry.file_name().to_string_lossy()))?;
    if filetype.is_dir() { crossterm::queue!(stdout, Print("/"))?; }
    crossterm::queue!(stdout, Print("\n\r"))?;
    // crossterm::queue!(stdout,
    //   Print(entry.file_name().to_string_lossy()),
    //   Print("\n\r"),
    // )?;
  };

  queue_color(Color::Reset, true)?;
  queue_color(Color::Reset, false)?;

  return Ok(());
}


