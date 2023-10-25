use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, PartialEq)]
pub enum ColorType {
    Dark,
    Light,
    Unknown,
}

impl TryFrom<String> for ColorType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return Ok(Self::from_str(value.as_str()).unwrap());
    }
}

impl FromStr for ColorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "255 255 255" => Self::Light,
            "000 000 000" => Self::Dark,
            "Light" => Self::Light,
            "Dark" => Self::Dark,
            _ => Self::Unknown,
        };
        Ok(t)
    }
}

#[derive(Serialize, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Outward,
    Return,
}

impl<'de> Deserialize<'de> for Direction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binding: String = String::deserialize(deserializer)?;
        let mode: &str = binding.as_str();
        Ok(Self::from_str(mode).unwrap())
    }
}

impl TryFrom<String> for Direction {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return Ok(Self::from_str(value.as_str()).unwrap());
    }
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "H" => Self::Outward,
            "R" => Self::Return,
            "Outward" => Self::Outward,
            "Return" => Self::Return,
            _ => panic!("Unknown direction: {}", s),
        };
        Ok(t)
    }
}
