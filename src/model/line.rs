use async_trait::async_trait;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

use super::types::ColorType;

#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
pub enum TransportMode {
    Underground,
    Bus,
    Funicular,
    Ship,
    Tramway,
    Rail,
    CableWay,
    Lift,
    Chairlift,
    RackRailroad,
    Unknown,
}

impl TransportMode {
    pub fn from_description(description: &LineDescription) -> Self {
        match description {
            LineDescription::Lift => Self::Lift,
            LineDescription::Bus => Self::Bus,
            LineDescription::PanoramaBus => Self::Bus,
            LineDescription::Nightbus => Self::Bus,
            LineDescription::NationalLongDistanceBus => Self::Bus,
            LineDescription::InternationalLongDistanceBus => Self::Bus,
            LineDescription::SemiFastBus => Self::Bus,
            LineDescription::ExpressBus => Self::Bus,
            LineDescription::IntercityBus => Self::Bus,
            LineDescription::Minibus => Self::Bus,
            LineDescription::NightBus => Self::Bus,
            LineDescription::LowFloorBus => Self::Bus,
            LineDescription::LowFloorTrolleyBus => Self::Bus,
            LineDescription::OnCallBus => Self::Bus,
            LineDescription::Taxi => Self::Bus,
            LineDescription::Chairlift => Self::Chairlift,
            LineDescription::RackRailroad => Self::RackRailroad,
            LineDescription::GondolaLift => Self::CableWay,
            LineDescription::Cableway => Self::CableWay,
            LineDescription::AerialTramway => Self::CableWay,
            LineDescription::Underground => Self::Underground,
            LineDescription::Funicular => Self::Funicular,
            LineDescription::Ship => Self::Ship,
            LineDescription::SteamShip => Self::Ship,
            LineDescription::FerryBoat => Self::Ship,
            LineDescription::Katamaran => Self::Ship,
            LineDescription::LowFloorTramway => Self::Tramway,
            LineDescription::Tramway => Self::Tramway,
            LineDescription::Nighttram => Self::Tramway,
            LineDescription::Aircraft => Self::Unknown,
            LineDescription::UnknownMode => Self::Unknown,
            LineDescription::Agencytrain => Self::Rail,
            LineDescription::Arco => Self::Rail,
            LineDescription::CarCarryingTrain => Self::Rail,
            LineDescription::CarTrain => Self::Rail,
            LineDescription::Altaria => Self::Rail,
            LineDescription::AltaVelocidadES => Self::Rail,
            LineDescription::BerninaExpress => Self::Rail,
            LineDescription::CityAirportTrain => Self::Rail,
            LineDescription::CityNightLine => Self::Rail,
            LineDescription::FastTrain => Self::Rail,
            LineDescription::SemiFastTrain => Self::Rail,
            LineDescription::EuroCity => Self::Rail,
            LineDescription::Euromed => Self::Rail,
            LineDescription::EuroNight => Self::Rail,
            LineDescription::EurostarItalia => Self::Rail,
            LineDescription::Eurostar => Self::Rail,
            LineDescription::SpecialTrain => Self::Rail,
            LineDescription::GlacierExpress => Self::Rail,
            LineDescription::InterCity => Self::Rail,
            LineDescription::InterCityExpress => Self::Rail,
            LineDescription::ICTiltingTrain => Self::Rail,
            LineDescription::InterCityNight => Self::Rail,
            LineDescription::InterRegio => Self::Rail,
            LineDescription::InterregioExpress => Self::Rail,
            LineDescription::Italo => Self::Rail,
            LineDescription::JailTrain => Self::Rail,
            LineDescription::EmptyMaterialTrain => Self::Rail,
            LineDescription::EmptyMaterialTrainWithPassengerTransport => Self::Rail,
            LineDescription::Nightjet => Self::Rail,
            LineDescription::NightTrain => Self::Rail,
            LineDescription::NoGuaranteedTrain => Self::Rail,
            LineDescription::PanoramaExpress => Self::Rail,
            LineDescription::Regio => Self::Rail,
            LineDescription::RegionalTrain => Self::Rail,
            LineDescription::RegioExpress => Self::Rail,
            LineDescription::Railjet => Self::Rail,
            LineDescription::RailjetXpress => Self::Rail,
            LineDescription::UrbanTrain => Self::Rail,
            LineDescription::NightUrbanTrain => Self::Rail,
            LineDescription::CityRailway => Self::Rail,
            LineDescription::Talgo => Self::Rail,
            LineDescription::TrainExpressRegional => Self::Rail,
            LineDescription::TER200 => Self::Rail,
            LineDescription::TrainGrandeVit => Self::Rail,
            LineDescription::Thalys => Self::Rail,
            LineDescription::TwojeLinieKolejowe => Self::Rail,
            LineDescription::UrlaubsExpress => Self::Rail,
            LineDescription::VoralpenExpress => Self::Rail,
            LineDescription::Westbahn => Self::Rail,
            LineDescription::InterConnex => Self::Rail,
            LineDescription::X2000TiltingTrain => Self::Rail,
            LineDescription::TrainCategoryUnknown => Self::Rail,
        }
    }
}

impl TryFrom<String> for TransportMode {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return Ok(Self::from_str(value.as_str()).unwrap());
    }
}

impl<'de> Deserialize<'de> for TransportMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binding: String = String::deserialize(deserializer)?;
        let mode: &str = binding.as_str();
        Ok(Self::from_str(mode).unwrap())
    }
}

impl FromStr for TransportMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t: Self = match s {
            "A" => Self::Lift,
            "B" => Self::Bus,
            "E" => Self::Chairlift,
            "H" => Self::RackRailroad,
            "L" => Self::CableWay,
            "M" => Self::Underground,
            "N" => Self::Funicular,
            "S" => Self::Ship,
            "T" => Self::Tramway,
            "U" => Self::Unknown,
            "Z" => Self::Rail,
            "Lift" => Self::Lift,
            "Bus" => Self::Bus,
            "Chairlift" => Self::Chairlift,
            "RackRailroad" => Self::RackRailroad,
            "CableWay" => Self::CableWay,
            "Underground" => Self::Underground,
            "Funicular" => Self::Funicular,
            "Ship" => Self::Ship,
            "Tramway" => Self::Tramway,
            "Unknown" => Self::Unknown,
            "Rail" => Self::Rail,
            _ => Self::Unknown,
        };
        Ok(t)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum LineDescription {
    Lift,
    Bus,
    Nightbus,
    PanoramaBus,
    NationalLongDistanceBus,
    InternationalLongDistanceBus,
    SemiFastBus,
    ExpressBus,
    IntercityBus,
    Minibus,
    NightBus,
    LowFloorBus,
    LowFloorTrolleyBus,
    OnCallBus,
    Taxi,
    Chairlift,
    RackRailroad,
    GondolaLift,
    Cableway,
    AerialTramway,
    Underground,
    Funicular,
    Ship,
    SteamShip,
    FerryBoat,
    Katamaran,
    LowFloorTramway,
    Tramway,
    Nighttram,
    Aircraft,
    UnknownMode,
    Agencytrain,
    Arco,
    CarCarryingTrain,
    CarTrain,
    Altaria,
    AltaVelocidadES,
    BerninaExpress,
    CityAirportTrain,
    CityNightLine,
    FastTrain,
    SemiFastTrain,
    EuroCity,
    Euromed,
    EuroNight,
    EurostarItalia,
    Eurostar,
    SpecialTrain,
    GlacierExpress,
    InterCity,
    InterCityExpress,
    ICTiltingTrain,
    InterCityNight,
    InterRegio,
    InterregioExpress,
    Italo,
    JailTrain,
    EmptyMaterialTrain,
    EmptyMaterialTrainWithPassengerTransport,
    Nightjet,
    NightTrain,
    NoGuaranteedTrain,
    PanoramaExpress,
    Regio,
    RegionalTrain,
    RegioExpress,
    Railjet,
    RailjetXpress,
    UrbanTrain,
    NightUrbanTrain,
    CityRailway,
    Talgo,
    TrainExpressRegional,
    TER200,
    TrainGrandeVit,
    Thalys,
    TwojeLinieKolejowe,
    UrlaubsExpress,
    VoralpenExpress,
    Westbahn,
    InterConnex,
    X2000TiltingTrain,
    TrainCategoryUnknown,
}

impl<'de> Deserialize<'de> for LineDescription {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binding: String = String::deserialize(deserializer)?;
        let desc: &str = binding.as_str();
        let t: Self = match desc {
            "ASC" => Self::Lift,
            "B" => Self::Bus,
            "BN" => Self::Nightbus,
            "BP" => Self::PanoramaBus,
            "BUS" => Self::Bus,
            "CAR" => Self::NationalLongDistanceBus,
            "CAX" => Self::InternationalLongDistanceBus,
            "EB" => Self::SemiFastBus,
            "EXB" => Self::ExpressBus,
            "ICB" => Self::IntercityBus,
            "KB" => Self::Minibus,
            "NB" => Self::NightBus,
            "NFB" => Self::LowFloorBus,
            "NFO" => Self::LowFloorTrolleyBus,
            "RUB" => Self::OnCallBus,
            "TX" => Self::Taxi,
            "SL" => Self::Chairlift,
            "CC" => Self::RackRailroad,
            "GB" => Self::GondolaLift,
            "LB" => Self::Cableway,
            "PB" => Self::AerialTramway,
            "M" => Self::Underground,
            "FUN" => Self::Funicular,
            "BAT" => Self::Ship,
            "BAV" => Self::SteamShip,
            "FAE" => Self::FerryBoat,
            "KAT" => Self::Katamaran,
            "NFT" => Self::LowFloorTramway,
            "T" => Self::Tramway,
            "TN" => Self::Nighttram,
            "AIR" => Self::Aircraft,
            "UUU" => Self::UnknownMode,
            "AG" => Self::Agencytrain,
            "ARC" => Self::Arco,
            "ARZ" => Self::CarCarryingTrain,
            "AT" => Self::CarTrain,
            "ATR" => Self::Altaria,
            "ATZ" => Self::CarTrain,
            "AVE" => Self::AltaVelocidadES,
            "BEX" => Self::BerninaExpress,
            "CAT" => Self::CityAirportTrain,
            "CNL" => Self::CityNightLine,
            "D" => Self::FastTrain,
            "E" => Self::SemiFastTrain,
            "EC" => Self::EuroCity,
            "EM" => Self::Euromed,
            "EN" => Self::EuroNight,
            "ES" => Self::EurostarItalia,
            "EST" => Self::Eurostar,
            "EXT" => Self::SpecialTrain,
            "GEX" => Self::GlacierExpress,
            "IC" => Self::InterCity,
            "ICE" => Self::InterCityExpress,
            "ICN" => Self::ICTiltingTrain,
            "IN" => Self::InterCityNight,
            "IR" => Self::InterRegio,
            "IRE" => Self::InterregioExpress,
            "IT" => Self::Italo,
            "JAT" => Self::JailTrain,
            "MAT" => Self::EmptyMaterialTrain,
            "MP" => Self::EmptyMaterialTrainWithPassengerTransport,
            "NJ" => Self::Nightjet,
            "NZ" => Self::NightTrain,
            "P" => Self::NoGuaranteedTrain,
            "PE" => Self::PanoramaExpress,
            "R" => Self::Regio,
            "RB" => Self::RegionalTrain,
            "RE" => Self::RegioExpress,
            "RJ" => Self::Railjet,
            "RJX" => Self::RailjetXpress,
            "S" => Self::UrbanTrain,
            "SN" => Self::NightUrbanTrain,
            "STB" => Self::CityRailway,
            "TAL" => Self::Talgo,
            "TER" => Self::TrainExpressRegional,
            "TE2" => Self::TER200,
            "TGV" => Self::TrainGrandeVit,
            "THA" => Self::Thalys,
            "TLK" => Self::TwojeLinieKolejowe,
            "UEX" => Self::UrlaubsExpress,
            "VAE" => Self::VoralpenExpress,
            "WB" => Self::Westbahn,
            "X" => Self::InterConnex,
            "X2" => Self::X2000TiltingTrain,
            "ZUG" => Self::TrainCategoryUnknown,
            _ => Self::UnknownMode,
        };
        Ok(t)
    }
}

#[derive(Serialize, FromRow, Debug)]
pub struct Line {
    pub id: i32,
    pub name: String,
    #[sqlx(try_from = "String")]
    pub color_type: ColorType,
    pub color: String,
}

#[async_trait]
impl Table for Line {
    const TABLE_NAME: &'static str = "lines";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.name.to_string()),
            Box::new(if self.color_type == ColorType::Unknown {
                String::new()
            } else {
                format!("{:?}", self.color_type)
            }),
            Box::new(self.color.to_string()),
        ]
    }

    fn keys() -> String {
        return "(id,name,color_type,color)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            name VARCHAR(10) NOT NULL,
            color_type VARCHAR(5),
            color VARCHAR(11)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
