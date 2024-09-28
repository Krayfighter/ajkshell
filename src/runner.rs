


// pub fn generate_run_prompt_string(command: &str, args: &[&str]) -> anyhow::Result<String> {

//   let output = await_command(command, args)?;
pub fn generate_run_prompt_string() -> anyhow::Result<String> {
  let output = await_command("prompt", &[])?;

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


pub fn run_command(command: &str, args: &[&str]) -> std::process::Command {
  let mut command = std::process::Command::new(command);
  command.stdin(std::process::Stdio::piped());
  command.stdout(std::process::Stdio::piped()); // this prevents double printing the command output

  for arg in args { command.arg(arg); }

  return command
}

pub fn await_command(command: &str, args: &[&str]) -> anyhow::Result<std::process::Output> {
  let mut command_handle = run_command(command, args);
  command_handle.spawn()
    .or_else(|e| Err(anyhow!("failed to spawn subprocess from executable: {command}\n\r{e}")))?
    .wait()
    .or_else(|e| Err(anyhow!("Error while waiting for subprocess: {command}\n\r{e}")))?;
  return command_handle.output()
    .map_err(|e| anyhow!("Failed to read stdout from subprocess: {command}\n\r{e}"));
}
