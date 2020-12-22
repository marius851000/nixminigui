use async_std::io::{ErrorKind, ReadExt};
use async_std::process::{Child, Command, Stdio};
use std::char::REPLACEMENT_CHARACTER;

pub struct AsyncCommand {
    child: Child,
    eof: bool,
    /// the output to the stdout, as a string
    output: String,
    /// the output of the stdout, but not yet decoded
    undecoded: Vec<u8>,
}

impl AsyncCommand {
    pub fn new(mut command: Command) -> Result<Self, async_std::io::Error> {
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(Self {
            child,
            eof: false,
            output: String::new(),
            undecoded: Vec::new(),
        })
    }

    async fn get_outputted_content(&mut self) -> Option<Vec<u8>> {
        if self.eof {
            return None;
        };
        //TODO: maybe add the ability to read multiple char at a time with a timeout
        let stdout = self.child.stdout.as_mut().unwrap();
        let mut output = vec![0u8; 1];
        match stdout.read_exact(&mut output).await {
            Ok(value) => value,
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => {
                    if !self.child.status().await.unwrap().success() {
                        panic!("the command exited with an error status");
                    };
                    self.eof = true;
                }
                _ => panic!(err),
            },
        };
        Some(output)
    }

    pub async fn update_and_get_log(&mut self) -> (&str, bool) {
        match self.get_outputted_content().await {
            Some(value) => {
                for char in value {
                    self.undecoded.push(char);
                    if self.undecoded.len() > 4 {
                        self.output.push(REPLACEMENT_CHARACTER);
                        self.undecoded.clear();
                    } else {
                        let tried_into_string = String::from_utf8(self.undecoded.clone());
                        match tried_into_string {
                            Ok(value) => {
                                self.output.push_str(&value);
                                self.undecoded.clear();
                            }
                            Err(_) => (),
                        };
                    };
                }
            }
            None => (),
        };
        (&self.output, self.eof)
    }
}
