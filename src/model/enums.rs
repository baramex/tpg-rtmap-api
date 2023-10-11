use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug)]
pub enum ColorType {
    Dark,
    Light
}

#[derive(Serialize, Eq, PartialEq, Debug)]
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
