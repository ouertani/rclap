use std::str::FromStr;

#[derive(Clone)]
pub struct Secret<S>(pub S);

impl<S> Secret<S> {
    pub fn new(value: S) -> Self {
        Self(value)
    }

    pub fn expose_secret(&self) -> &S {
        &self.0
    }
}

impl<S> std::fmt::Display for Secret<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl<S> std::fmt::Debug for Secret<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl<S: FromStr> std::str::FromStr for Secret<S> {
    type Err = S::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = S::from_str(s)?;
        Ok(Secret::new(value))
    }
}

impl<S: PartialEq> PartialEq for Secret<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(feature = "serde")]
impl<C> serde::Serialize for Secret<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("*****")
    }
}

#[cfg(feature = "serde")]
impl<'de, S: serde::Deserialize<'de>> serde::Deserialize<'de> for Secret<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: S = S::deserialize(deserializer)?;
        Ok(Secret::new(s))
    }
}
