use std::error;
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::path::Path;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    FileError(String, std::io::Error),
    YamlError(String, serde_yaml::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FileError(ref filename, ref err) =>
                write!(f, "Failed to open \"{filename}\": {err}"),
            Error::YamlError(ref filename, ref err) =>
                write!(f, "Failed to parse \"{filename}\": {err}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::FileError(_, ref e) => Some(e),
            Error::YamlError(_, ref e) => Some(e),
        }
    }
}

pub trait YamlRW: Serialize + DeserializeOwned {
    fn read_yaml<P: Copy + Display + AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(serde_yaml::from_reader(File::open(path).map_err(|err| Error::FileError(path.to_string(), err))?).map_err(|err| Error::YamlError(path.to_string(), err))?)
    }

    fn write_yaml<P: Copy + Display + AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        serde_yaml::to_writer(File::create(path).map_err(|err| Error::FileError(path.to_string(), err))?, &self).unwrap();
        Ok(())
    }
}
