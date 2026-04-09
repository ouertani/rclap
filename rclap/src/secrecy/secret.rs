use std::str::FromStr;

use secrecy::{CloneableSecret, ExposeSecret, SecretBox};

#[derive(Clone)]
#[repr(transparent)]
pub struct Secret<S: CloneableSecret>(pub SecretBox<S>);

impl<S: CloneableSecret> Secret<S> {
    pub fn new(value: S) -> Self {
        Self(SecretBox::new(Box::new(value)))
    }

    pub fn expose_secret(&self) -> &S {
        self.0.expose_secret()
    }
}

impl<S: CloneableSecret> std::fmt::Display for Secret<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl<S: CloneableSecret> std::fmt::Debug for Secret<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl<S: CloneableSecret + FromStr> std::str::FromStr for Secret<S> {
    type Err = S::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = S::from_str(s)?;
        Ok(Secret::new(value))
    }
}
impl<S: CloneableSecret> From<S> for Secret<S> {
    fn from(value: S) -> Self {
        Secret::new(value)
    }
}

impl<S: CloneableSecret + PartialEq> PartialEq for Secret<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(feature = "serde")]
impl<C: CloneableSecret> serde::Serialize for Secret<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("*****")
    }
}

#[cfg(feature = "serde")]
impl<'de, S: CloneableSecret + serde::Deserialize<'de>> serde::Deserialize<'de> for Secret<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: S = S::deserialize(deserializer)?;
        Ok(Secret::new(s))
    }
}
