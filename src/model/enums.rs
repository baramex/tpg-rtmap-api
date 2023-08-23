use serde::{Serialize, Deserializer, Deserialize};

#[derive(Serialize, Eq, PartialEq, Debug)]
pub enum Direction {
    Outward,
    Return
}

impl<'de> Deserialize<'de> for Direction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binding: String = String::deserialize(deserializer)?;
        let mode: &str = binding.as_str();
        let t: Self = match mode {
            "H" => Self::Outward,
            "R" => Self::Return,
            _ => panic!("Unknown direction: {}", mode),
        };
        Ok(t)
    }
}