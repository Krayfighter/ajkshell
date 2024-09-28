use std::marker::PhantomData;









pub fn change_directory(statement: &[&str]) -> anyhow::Result<()> {
  if statement.len() > 2 { bail!("Too many arguments (max 1): {:?}", &statement[1..statement.len()-1]) }
  if statement.len() == 1 { bail!("Missing positional argument: (dir)"); }

  std::env::set_current_dir(statement[1])?;

  return Ok(());
}

