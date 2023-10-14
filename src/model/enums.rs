use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug)]
pub enum ColorType {
    Dark,
    Light
}

impl FromStr for ColorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "255 255 255" => Self::Light,
            "000 000 000" => Self::Dark,
            _ => panic!("Unknown color type: {}", s),
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

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "H" => Self::Outward,
            "R" => Self::Return,
            _ => panic!("Unknown direction: {}", s),
        };
        Ok(t)
    }
}