

#[derive(Debug)]
pub enum Token {
  Command(crate::SliceTree<String>),
  Argument(crate::SliceTree<String>),
  Pipe,
}
impl Token {
  pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
    return match self {
      Token::Command(slice) => slice.as_slice(),
      Token::Argument(slice) => slice.as_slice(),
      _ => b"|",
    }
  }
}

fn end_token(
  src: &crate::SliceTree<String>,
  start: usize,
  end: usize,
  is_command: &mut bool,
) -> Token {
  let token_slice = src.subslice(start..end);
  let token = match *is_command {
    true => Token::Command(token_slice),
    false => Token::Argument(token_slice),
  };
  if *is_command { *is_command = false; }
  return token;
}

pub fn lex(src: &str) -> anyhow::Result<Vec<Token>> {
  let distributed_source = crate::SliceTree::consume(String::from(src));

  let mut tokens = Vec::<Token>::new();

  let mut token_start: Option<usize> = Some(0);
  let mut next_token_is_command = true;

  let mut inside_quotes = false;

  for (index, chr) in src.chars().enumerate() {
    if inside_quotes { // if inside a quote block, iqnore all special characters
      if chr == '\"' {
        tokens.push( end_token(
          &distributed_source, token_start.unwrap(), index, &mut next_token_is_command
        ) );
      } else { if let None = token_start { token_start = Some(index); } }
      continue
    }
    match chr {
      // '$' => {},
      '\"' => {
        if let Some(_) = token_start { bail!("invalid quote placement"); }
        inside_quotes = true;
      }, // toggle quote mode
      ' ' => {
        if let Some(start) = token_start {
          tokens.push( end_token(
            &distributed_source, start, index, &mut next_token_is_command
          ) );
          token_start = None;
        }
      },
      '|' => {
        if token_start.is_some() { bail!("Found Pipe inside token as column {}", index) }
        tokens.push(Token::Pipe);
        next_token_is_command = true;
      },
      _ => {
        if token_start.is_none() { token_start = Some(index); }
      },
    }
  }
  if let Some(start) = token_start {
    tokens.push(end_token(&distributed_source, start, src.len(), &mut next_token_is_command));
  }
  return Ok(tokens);
}



pub struct ParsedCommand {
  command: crate::SliceTree<String>,
  // this can't be a Vec<SliceTree<_>> because
  // it may contain subcommands that need to be evaluated
  // so I created a type that has lazy execution that results
  // in a Box<[u8]>
  args: Vec<crate::ReprAsOsStr>
}

impl crate::ReprAs<Box<[u8]>> for ParsedCommand {
  fn repr_as(&self) -> Box<[u8]> {
    todo!();
  }
}

// This is needed from std::rc::Rc::from(ParsedCommand) to be implemented
impl crate::ReprAsOsStrTrait for ParsedCommand { }

impl ParsedCommand {
  pub fn new(statement: &[Token]) -> anyhow::Result<ParsedCommand> {
    if statement.len() == 0 { crate::interface::expect_log_error(anyhow!("Err: Statement of length 0")) }
    // if statement[0] == Token::Pipe { panic!("command starts with pipe ?not possible"); }
    let command = match &statement[0] {
        Token::Command(cmd) => cmd,
        Token::Argument(_) | Token::Pipe => panic!("Invalid State"),
    };
    if statement.len() == 1 { return Ok( Self{ command: command.clone(), args: vec!() } ) }
    else {
      let mut args = Vec::<crate::ReprAsOsStr>::new();
      for (index, arg) in statement[1..statement.len()].iter().enumerate() {
        match arg {
          Token::Command(c) => {
            // TODO: this is to be a subommand within 
            args.push( std::rc::Rc::from(ParsedCommand::new(statement)?) );
            break;
          },
          Token::Argument(a) => {
            args.push( std::rc::Rc::from(a.clone()) )
            // args.push(std::rc::Rc::new((*a).to_owned()))
          },
          Token::Pipe => panic!("Invalid Pipe in Arguments"),
        }
      }
      return Ok( Self {
        command: command.clone(), args
        // command: statement[0].as_str().to_owned(), args
      })
    }
  }
  pub fn build(&self,
    stdout: Option<std::process::Stdio>
  ) -> std::process::Command {
    return match self.command.as_ref() {
      b"cd" => todo!(),
      b"exit" => todo!(),
      // b"cd" => Box::new(crate::builtins::ChangeDirectory::new(self.args.clone())),
      // b"exit" => Box::new(crate::builtins::Exit::new(self.args.clone())),
      // cmd => Box::new(crate::runner::build_command(crate::utils::as_str(cmd), &self.args))
      // cmd => crate::runner::OsCommand::spawn(crate::utils::as_str(cmd))
      cmd => {
        let mut command = std::process::Command::new(
          crate::utils::as_str(cmd)
        );
        for arg in self.args.iter() {
          command.arg(crate::utils::as_os_str(arg.repr_as().as_ref()));
        }
        match stdout {
          Some(stdout) => command.stdin(stdout),
          None => command.stdin(std::process::Stdio::piped())
        };
        command.stdout(std::process::Stdio::piped())
          .stderr(std::process::Stdio::piped());

        command
      }
    }
    // let args = self.args.clone().into_iter()
    //   .map(|arg| (arg.as_ref())
    //   .collect::<Vec<String>>();

    // return runner::build_command(&self.command, &self.args);
  }
}

pub fn parse(tokens: &[Token]) -> anyhow::Result<Vec<ParsedCommand>> {

  // seperate by pipes

  // segments of the full command sperated by pipes
  let mut pipe_segments: Vec<&[Token]> = Vec::new();
  let mut current_expr_start: Option<usize> = None;

  for (index, token) in tokens.iter().enumerate() {
    if let Token::Pipe = token {
      match current_expr_start {
        Some(start) => {
          pipe_segments.push(&tokens[start..index]);
          current_expr_start = None
        },
        None => bail!("Err: Double Pipe")
      }
    } else { if let None = current_expr_start {
      current_expr_start = Some(index)
    } }
  }
  if let Some(start) = current_expr_start {
    pipe_segments.push(&tokens[start..tokens.len()]);
  }else { panic!("Err: Command ends in pipe (pipe from nowhere)") }

  println!("Segments of command: {:?}", pipe_segments);

  // organize subcommands

  return Ok( pipe_segments.into_iter()
    .map(|segment| ParsedCommand::new(segment).unwrap())
    .collect::<Vec<ParsedCommand>>() );
}


