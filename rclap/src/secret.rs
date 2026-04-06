use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, PartialEq)]
pub struct Secret<T>(T);

impl Display for Secret<String> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "****************")
    }
}
impl fmt::Debug for Secret<String> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "****************")
    }
}

impl FromStr for Secret<String> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Secret(s.to_string()))
    }
}

impl Secret<String> {
    pub fn new(value: &str) -> Self {
        Secret(value.to_string())
    }
    pub fn expose_secret(&self) -> String {
        self.0.clone()
    }
}

pub trait IntoSecret<T> {
    fn into_secret(self) -> Secret<T>;
}

impl IntoSecret<String> for &str {
    fn into_secret(self) -> Secret<String> {
        Secret(self.to_string())
    }
}
