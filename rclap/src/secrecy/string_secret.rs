use secrecy::{ExposeSecret, SecretString as SS};

#[derive(Clone)]
#[repr(transparent)]
pub struct StringSecret(pub SS);

impl StringSecret {
    pub fn new(value: &str) -> Self {
        Self(SS::from(value.to_string()))
    }

    pub fn expose_secret(&self) -> String {
        self.0.expose_secret().to_string()
    }
}

impl std::fmt::Display for StringSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl std::fmt::Debug for StringSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl std::str::FromStr for StringSecret {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(SS::from(s.to_string())))
    }
}

impl PartialEq for StringSecret {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for StringSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("*****")
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StringSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(StringSecret::new(&s))
    }
}
