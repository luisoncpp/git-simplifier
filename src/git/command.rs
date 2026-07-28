use std::collections::BTreeMap;
use std::ffi::OsString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessMode {
    ReadOnly,
    Write,
}

pub struct GitCommand {
    pub args: Vec<OsString>,
    pub access: AccessMode,
    pub environment: BTreeMap<OsString, OsString>,
    pub stdin: Option<Vec<u8>>,
}

impl GitCommand {
    pub fn read(args: Vec<OsString>) -> Self {
        Self::new(args, AccessMode::ReadOnly)
    }

    pub fn write(args: Vec<OsString>) -> Self {
        Self::new(args, AccessMode::Write)
    }

    pub fn with_environment(mut self, key: OsString, value: OsString) -> Self {
        self.environment.insert(key, value);
        self
    }

    pub fn with_stdin(mut self, input: Vec<u8>) -> Self {
        self.stdin = Some(input);
        self
    }

    fn new(args: Vec<OsString>, access: AccessMode) -> Self {
        Self {
            args,
            access,
            environment: BTreeMap::new(),
            stdin: None,
        }
    }
}

pub struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
}
