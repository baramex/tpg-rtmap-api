use crate::model::{
    enums::Direction,
    line::{Line, TransportMode},
};
use derive_more::Display;
use fixed_width::{FieldSet, FixedWidth, LineBreak, Reader};
use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};

pub struct HRDF {
    pub directory: PathBuf,
    pub agency_id: u16,
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanZ {
    #[fixed_width(range = "3..9")]
    jouney_number: u32,
    #[fixed_width(range = "10..17")]
    agency_id: u16,
    #[fixed_width(range = "19..22")]
    option_count: u16,
}

impl RawFahrplanZ {
    const IDENTIFIER: &str = "*Z";
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanG {
    #[fixed_width(range = "4..7")]
    transport_mode: TransportMode,
    #[fixed_width(range = "8..15")]
    origin: u32,
    #[fixed_width(range = "16..23")]
    destination: u32,
}

impl RawFahrplanG {
    const IDENTIFIER: &str = "*G";
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanA {
    #[fixed_width(range = "7..14")]
    origin: u32,
    #[fixed_width(range = "15..22")]
    destination: u32,
    #[fixed_width(range = "23..29")]
    bit_field_number: u32,
}

impl RawFahrplanA {
    const IDENTIFIER: &str = "*A VE";
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanL {
    #[fixed_width(range = "4..12")]
    line_number: u32,
    #[fixed_width(range = "13..20")]
    origin: u32,
    #[fixed_width(range = "21..28")]
    destination: u32,
}

impl RawFahrplanL {
    const IDENTIFIER: &str = "*L";
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanR {
    #[fixed_width(range = "4..5")]
    direction: Direction,
    #[fixed_width(range = "7..13")]
    direction_number: u32,
    #[fixed_width(range = "14..21")]
    origin: u32,
    #[fixed_width(range = "22..29")]
    destination: u32,
}

impl RawFahrplanR {
    const IDENTIFIER: &str = "*R";
}

#[derive(FixedWidth, Deserialize, Debug)]
pub struct RawFahrplanStop {
    #[fixed_width(range = "0..7")]
    stop_id: u32,
    #[fixed_width(range = "8..28")]
    name: String,
    #[fixed_width(range = "30..35")]
    arrival_time: u16,
    #[fixed_width(range = "37..42")]
    departure_time: u16,
}

#[derive(Debug)]
pub struct Fahrplan {
    pub z: RawFahrplanZ,
    pub g: RawFahrplanG,
    pub a: RawFahrplanA,
    pub l: RawFahrplanL,
    pub r: RawFahrplanR,
    pub stops: Vec<RawFahrplanStop>,
}

/*
fichiers hrdf
fplan:
*Z journey_number TU_CODE n_intervals
*G VM_ART ori des
*A VE ori des bit_field_number
*L #LINIE ori des
*R direction direction_number ori des
STOP_ID name arrival_time departure_time
...

linie:
LINIE field values...
field = K:name|N:T name|F:colortype|B:color

feiertag ?:
jours fériés

bitfield:
bit_field_number bit_field
bitfield: 1 hexa = 4 bits, 4 bits = 4 days (0|1)
2firsts and 2 lasts are inserted

gleise: platform info
*/

impl HRDF {
    fn create_reader(&self, filename: &str) -> Result<Reader<File>, fixed_width::Error> {
        let path: PathBuf = self.directory.join(filename);
        Reader::from_file(path)
    }

    pub fn get_fahrplans(&self) -> Result<Vec<Fahrplan>, fixed_width::Error> {
        let mut reader: Reader<File> = self.create_reader("FPLAN")?;
        println!("Reader created");

        let mut fplans: Vec<Fahrplan> = Vec::new();

        while let Some(Ok(bytes)) = reader.next_record() {
            if bytes.starts_with(RawFahrplanZ::IDENTIFIER.as_bytes()) {
                println!("Found a fplan");
                let line_z: RawFahrplanZ = fixed_width::from_bytes(bytes)?;

                if line_z.agency_id != self.agency_id {
                    continue;
                }

                let mut line_g: Option<RawFahrplanG> = None;
                let mut line_a: Option<RawFahrplanA> = None;
                let mut line_l: Option<RawFahrplanL> = None;
                let mut line_r: Option<RawFahrplanR> = None;
                let mut stops: Vec<RawFahrplanStop> = Vec::new();

                while let Some(Ok(bytes2)) = reader.next_record() {
                    if bytes2.starts_with(RawFahrplanG::IDENTIFIER.as_bytes()) {
                        line_g = Some(fixed_width::from_bytes(bytes2)?);
                    } else if bytes2.starts_with(RawFahrplanA::IDENTIFIER.as_bytes()) {
                        line_a = Some(fixed_width::from_bytes(bytes2)?);
                    } else if bytes2.starts_with(RawFahrplanL::IDENTIFIER.as_bytes()) {
                        line_l = Some(fixed_width::from_bytes(bytes2)?);
                    } else if bytes2.starts_with(RawFahrplanR::IDENTIFIER.as_bytes()) {
                        line_r = Some(fixed_width::from_bytes(bytes2)?);
                    } else {
                        stops.push(fixed_width::from_bytes(bytes2)?);
                    }
                }

                if line_g.is_none() || line_a.is_none() || line_l.is_none() || line_r.is_none() {
                    continue;
                }

                let fplan: Fahrplan = Fahrplan {
                    z: line_z,
                    g: line_g.unwrap(),
                    a: line_a.unwrap(),
                    l: line_l.unwrap(),
                    r: line_r.unwrap(),
                    stops,
                };

                println!("{:#?}", fplan);
                fplans.push(fplan);
            }
        }

        return Ok(fplans);
    }
}
