use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug)]
pub struct Hour {
    pub hour: i16,
    pub minute: i16,
}

impl Hour {
    pub fn value(&self) -> i16 {
        self.hour * 60 + self.minute
    }

    pub fn from_value(value: i16) -> Self {
        Self {
            hour: value / 60,
            minute: value % 60,
        }
    }
}

impl FromStr for Hour {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hour: i16 = s[1..3].parse().unwrap();
        let minute: i16 = s[3..5].parse().unwrap();
        Ok(Self { hour, minute })
    }
}

impl Serialize for Hour {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        serializer.serialize_i16(self.value())
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub enum ColorType {
    Dark,
    Light,
    Unknown
}

impl FromStr for ColorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "255 255 255" => Self::Light,
            "000 000 000" => Self::Dark,
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