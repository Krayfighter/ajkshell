

pub type Builtin = &'static dyn FnMut<
  (crate::SliceTree<String>,),
  Output=anyhow::Result<String>
>;


struct BuiltinChild {
}


fn empty_output() -> std::process::Output {
  return std::process::Output{
    stdout: vec!(), stderr: vec!(),
    status: std::process::ExitStatus::default(),
  }
}


pub struct ChangeDirectory {
  // statement: Vec<std::rc::Rc<dyn AsRef<std::ffi::OsStr>>>
  statement: Vec<crate::ReprAsOsStr>
}
impl crate::runner::Runnable for ChangeDirectory {
  fn run(&mut self) -> anyhow::Result<std::process::Output> {
    return match change_directory(&self.statement) {
      Ok(()) => Ok(empty_output()),
      Err(e) => Err(e),
    };
  }
}
impl ChangeDirectory {
  pub fn new( statement: Vec<crate::ReprAsOsStr>) -> ChangeDirectory {
    return ChangeDirectory { statement };
  }
}

fn change_directory(statement: &[crate::ReprAsOsStr]) -> anyhow::Result<()> {
  // if statement.len() > 2 { bail!("Too many arguments (max 1): {:?}", &statement[1..statement.len()-1]) }
  if statement.len() > 2 { bail!("Too many arguments") }
  if statement.len() == 1 { bail!("Missing positional argument: (dir)"); }

  std::env::set_current_dir::<&std::ffi::OsStr>(
    // this is ok since we know that the source of these bytes is a string, and that
    // &[u8] and &std::ffi::OsStr have the same memory layout
    unsafe{ std::mem::transmute(statement[0].as_ref().repr_as().as_ref()) }
  )?;

  return Ok(());
}

pub struct Exit { statement: Vec<crate::ReprAsOsStr> }

impl crate::runner::Runnable for Exit {
  fn run(&mut self) -> anyhow::Result<std::process::Output> {
    return match exit(&self.statement) {
      Ok(()) => Ok(empty_output()),
      Err(e) => Err(e),
    }
  }
}

impl Exit {
  pub fn new(statement: Vec<crate::ReprAsOsStr>) -> Self {
    return Self { statement }
  }
}

fn exit(statement: &[crate::ReprAsOsStr]) -> anyhow::Result<()> {
  if statement.len() > 1 {
    bail!("Unexpected argument(s)");
    // bail!("Unexpected argument(s): {:?}", &statement[1..statement.len()])
  }
  return Ok(());
}

