

// pub enum RunError {
//   Command(std::process::ExitStatus),
//   Builtin(anyhow::Error),
//   None
// }

// type RunnableArg<'a> = &'a [&'a dyn AsRef<std::ffi::OsStr>]



enum ChildProcess {
  Real(std::process::Child),
  Builtin()
}

// TODO: make a runnable type that can be stored in memory
// and run / spawned and await'd at a later time



enum RunnableTarget {
  Command(std::process::Command),
  Builtin(crate::builtins::Builtin)
  // Builtin(Box<
  //   dyn FnMut<
  //     (Vec<crate::ReprAsOsStr>,),
  //     Output=anyhow::Result<String>
  //   >
  // >)
  // Builtin(std::rc::Rc<dyn FnMut<(RunnableArg,), Output=Result<String>>>)
}

use std::rc::Rc;

struct Runnable {
  target: RunnableTarget,
  stdin: Option<Rc<dyn std::io::Read>>,
  stdout: Option<Rc<dyn std::io::Write>>,
  stderr: Option<Rc<dyn std::io::Write>>,
}

impl Runnable {
  pub fn new_command(source: std::process::Command) -> Self {
    return Self {
      target: RunnableTarget::Command(source),
      stdin: None, stdout: None, stderr: None
    }
  }
  pub fn new_builtin(source: crate::builtins::Builtin) -> Self {
    return Self {
      target: RunnableTarget::Builtin(source),
      stdin: None, stdout: None, stderr: None
    }
  }
  // pub fn stdin<T: std::io::Read>(&mut self, input: T) {
  //   self.stdin = Rc::from(input);
  // }
  fn spawn_and_wait(&mut self) -> anyhow::Result<()> {
    match self.target {
      RunnableTarget::Command(cmd) => {},
      RunnableTarget::Builtin(cmd) => {
        let result = (cmd)()
      },
    }

    todo!()
  }
}


impl RunnableTarget {
  fn spawn(&mut self) -> std::process::Child {
  }
}

pub trait Runnable {
  // fn run(&mut self) -> anyhow::Result<(
  //   Vec<u8>, Vec<u8>, RunError
  // )>;
  fn run(&mut self) -> anyhow::Result<std::process::Output>;
}

impl Runnable for std::process::Command {
  fn run(&mut self) -> anyhow::Result<std::process::Output> {
    // return self.output()
    return match self.output() {
      // Ok(output) => Ok(output),
      Ok(output) => { Ok(output)
        // Ok((
        //   output.stdout,
        //   output.stderr,
        //   RunError::Command(output.status)
        // ))
      }
      Err(e) => bail!("failed to run command: {e}")
    }
  }
}


pub fn generate_run_prompt_string() -> anyhow::Result<String> {
  let output = await_command::<&str>("prompt", &[])?;

  if output.stderr.as_slice().len() > 0 {
    bail!("Error in run prompt: {}",
      unsafe { output.stdout.as_ascii_unchecked().as_str() }
    )
  }

  return Ok(String::from(
    output.stdout.as_ascii()
      .ok_or(anyhow!("command returned non-ascii"))?
      .as_str()
  ));
}


pub fn build_command(command: &str, args: &[crate::ReprAsOsStr]) -> std::process::Command {
  let mut command = std::process::Command::new(command);
  command.stdin(std::process::Stdio::piped());
  command.stdout(std::process::Stdio::piped()); // this prevents double printing the command output

  for arg in args { command.arg(
    crate::utils::as_os_str(arg.repr_as().as_ref())
    // // this is ok since &[u8] and std::ffi::OsStr have
    // // the same memory layout
    // unsafe{ std::mem::transmute::<&[u8], &std::ffi::OsStr>(arg.repr_as().as_ref()) }
  ); }

  return command
}

pub fn await_command(command: &str, args: &[crate::ReprAsOsStr]) -> anyhow::Result<std::process::Output> {
  let mut command_handle = build_command(command, args);
  command_handle.spawn()
    .or_else(|e| Err(anyhow!("failed to spawn subprocess from executable: {command}\n\r{e}")))?
    .wait()
    .or_else(|e| Err(anyhow!("Error while waiting for subprocess: {command}\n\r{e}")))?;
  return command_handle.output()
    .map_err(|e| anyhow!("Failed to read stdout from subprocess: {command}\n\r{e}"));
}
