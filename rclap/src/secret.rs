#[cfg(feature = "secret")]
use secrecy::{ExposeSecret, SecretString};

#[cfg(feature = "secret")]
#[derive(Clone)]
pub struct Secret(pub SecretString);

#[cfg(feature = "secret")]
impl Secret {
    pub fn new(value: &str) -> Self {
        Self(SecretString::from(value.to_string()))
    }

    pub fn expose_secret(&self) -> String {
        self.0.expose_secret().to_string()
    }
}

#[cfg(feature = "secret")]
impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

#[cfg(feature = "secret")]
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

#[cfg(feature = "secret")]
impl std::str::FromStr for Secret {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(SecretString::from(s.to_string())))
    }
}

#[cfg(feature = "secret")]
impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(feature = "secret")]
impl serde::Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("*****")
    }
}

#[cfg(feature = "secret")]
impl<'de> serde::Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Secret::new(&s))
    }
}

