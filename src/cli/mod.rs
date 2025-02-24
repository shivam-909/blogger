use std::{collections::HashMap, env, path::Path};

use crate::{
    backend::codegen::Generator,
    errors::BloggerError,
    fs,
    lexer::{lexer::Lexer, tokens::token_specs},
    parser::parser::Parser,
    regex::matcher::Matcher,
};

#[derive(Debug)]
struct Flags {
    inner: HashMap<String, Option<String>>,
}

impl Flags {
    fn new() -> Self {
        Flags {
            inner: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, value: Option<String>) {
        self.inner.insert(key, value);
    }

    fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key).and_then(|v| v.as_ref())
    }

    fn must(&self, keys: &[&str]) -> Result<(), BloggerError> {
        keys.iter().try_for_each(|key| {
            if !self.contains(key) {
                return Err(BloggerError::CommandError(format!("expected flag {}", key)));
            }

            Ok(())
        })
    }
}

trait Command {
    fn run(&self, args: &[String], flags: &Flags) -> Result<(), BloggerError>;
}

macro_rules! new_command {
    ($name:ident, $cmd_name:expr, ($param:ident, $param2:ident) $run:block) => {
        struct $name;

        impl Command for $name {
            fn run(&self, $param: &[String], $param2: &Flags) -> Result<(), BloggerError> {
                {
                    $run
                }
            }
        }
    };
}

new_command!(LexCommand, "tokenises input and outputs token list", (_args, flags) {
    flags.must(&vec!["--src"])?;
    let src_location = flags.get("--src").unwrap();
    let src_path = Path::new(src_location);
    let src_content = fs::read_file_to_string(src_path)?;
    let lexer = Lexer::new(&src_content,token_specs());
    for token in lexer {
        match token {
            Ok(spanned_tok) => {
                println!("{:?}", spanned_tok.kind);
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    Ok(())
});

new_command!(ParseCommand,"tokenises and parses input, outputs AST", (_args, flags) {
    flags.must(&vec!["--src"])?;
    let src_location = flags.get("--src").unwrap();
    let src_path = Path::new(src_location);
    let src_content = fs::read_file_to_string(src_path)?;
    let lexer = Lexer::new(&src_content,token_specs());
    let parser = Parser::new(lexer,&src_content).parse()?;
    println!("{:#?}", parser);
    Ok(())
});

new_command!(CompileCommand, "compiles input into blog output", (_args, flags) {
    flags.must(&vec!["--src", "--dst"])?;

    let src_location = flags.get("--src").unwrap();
    let src_path = Path::new(src_location);

    let dst_location = flags.get("--dst").unwrap();
    let dst_path = Path::new(dst_location);

    let src_content = fs::read_file_to_string(src_path)?;
    let mut dst_buf = fs::create_write_buffer(dst_path)?;

    let lexer = Lexer::new(&src_content,token_specs());
    let mut parser = Parser::new(lexer,&src_content);
    let program = parser.parse()?;
    let mut compiler = Generator::new(program);
    compiler.compile(&mut dst_buf)?;
    Ok(())
});

fn parse_flags(args: &[String]) -> Flags {
    let m = Matcher::new(r"(-.-).([a-z]*).=.(([a-z]|/|\.|_)*)").unwrap();
    let mut f = Flags::new();
    for a in args {
        if m.matches(a) {
            let halves: Vec<&str> = a.split("=").collect();
            assert_eq!(
                halves.len(),
                2,
                "flag format must have two halves separated by ="
            );
            f.insert(halves[0].to_string(), Some(halves[1].to_string()));
        }
    }
    f
}

pub fn run() -> Result<(), BloggerError> {
    let args: Vec<String> = env::args().skip(1).collect();
    let flags = parse_flags(&args);
    let command = args[0].clone();

    let command: Box<dyn Command> = match command.as_str() {
        "lex" => Box::new(LexCommand),
        "compile" => Box::new(CompileCommand),
        "parse" => Box::new(ParseCommand),
        _ => {
            return Err(BloggerError::CommandError(format!(
                "unknown command: {}",
                command
            )))
        }
    };

    command.run(&args, &flags)
}
