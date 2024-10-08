


pub fn as_os_str<'a>(source: &'a [u8]) -> &'a std::ffi::OsStr {
  return unsafe { std::mem::transmute(
    source.as_ref()
  ) }
}

// NOTE this may cause undefined behavior
// if there are strange (maybe unicode) contents
// in the source slice
pub fn as_str<'a>(source: &'a [u8]) -> &'a str {
  return unsafe { std::mem::transmute( source ) }
}




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

