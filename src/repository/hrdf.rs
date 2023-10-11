use unicode_segmentation::UnicodeSegmentation;

use crate::model::{
    enums::{Direction, ColorType},
    line::{Line, TransportMode}, bitfield::Bitfield,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Error, Lines},
    path::PathBuf,
    cmp
};

pub struct HRDF {
    pub directory: PathBuf,
    pub agency_id: u16,
}

macro_rules! define_record {
    (
        $record_name:ident {
            $(
                $field_name:ident : $field_type:ty => $start:expr => $end:expr
            ),* $(,)?
        }
    ) => {
        #[derive(Debug)]
        pub struct $record_name {
            $(
                $field_name: $field_type,
            )*
        }

        impl $record_name {
            pub fn from_line(line: &str) -> Self {
                $(
                    let chars: Vec<&str> = UnicodeSegmentation::graphemes(line, true).collect();
                    let $field_name: $field_type = chars[$start..cmp::min($end, chars.len())].join("").to_string().trim().parse::<$field_type>().unwrap_or_else(|_| panic!("Failed to parse field {} from {}", stringify!($field_name), line[$start..cmp::min($end, chars.len())].to_string()));
                )*
                $record_name {
                    $(
                        $field_name,
                    )*
                }
            }
        }
    }
}

define_record! {
    RawFahrplanZ {
        journey_number: u32 => 3 => 9,
        agency_id: u16 => 10 => 16,
        option_count: u16 => 19 => 22,
    }
}

define_record! {
    RawFahrplanG {
        transport_mode: TransportMode => 3 => 6,
        origin: u32 => 7 => 14,
        destination: u32 => 15 => 22,
    }
}

define_record! {
    RawFahrplanA {
        origin: u32 => 6 => 13,
        destination: u32 => 14 => 21,
        bit_field_number: u32 => 22 => 28,
    }
}

define_record! {
    RawFahrplanL {
        line_number: u32 => 4 => 11,
        origin: u32 => 12 => 19,
        destination: u32 => 20 => 27,
    }
}

define_record! {
    RawFahrplanR {
        direction: Direction => 3 => 4,
        direction_number: u32 => 6 => 12,
        origin: u32 => 13 => 20,
        destination: u32 => 21 => 28,
    }
}

define_record! {
    RawFahrplanStop {
        stop_id: u32 => 0 => 7,
        name: String => 8 => 28,
        arrival_time: String => 30 => 35,
        departure_time: String => 37 => 42,
    }
}

define_record! {
    RawLinieN {
        number: u32 => 0 => 7,
        name: String => 12 => 22,
    }
}

define_record! {
    RawLinieF {
        number: u32 => 0 => 7,
        color_type: ColorType => 10 => 21,
    }
}

define_record! {
    RawLinieB {
        number: u32 => 0 => 7,
        color: String => 10 => 21,
    }
}

define_record! {
    RawBitfeld {
        number: u32 => 0 => 6,
        days: String => 7 => 99,
    }
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

bitfield:
bit_field_number bit_field
bitfield: 1 hexa = 4 bits, 4 bits = 4 days (0|1)
2firsts and 2 lasts are inserted

gleise: platform info
?: line path information ?
*/

impl HRDF {
    fn create_reader(&self, filename: &str) -> Result<BufReader<File>, Error> {
        let path: PathBuf = self.directory.join(filename);
        let reader: BufReader<File> = BufReader::new(File::open(path)?);

        return Ok(reader);
    }

    pub fn get_bitfields(&self) -> Result<Vec<Bitfield>, Error> {
        let reader: BufReader<File> = self.create_reader("BITFELD")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut bitfields: Vec<Bitfield> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let bf_line: RawBitfeld = RawBitfeld::from_line(&line);

            let bitfield: Bitfield = Bitfield {
                id: bf_line.number,
                days: bf_line.days,
            };

            bitfields.push(bitfield);
        }

        return Ok(bitfields);
    }

    pub fn get_lines(&self) -> Result<Vec<Line>, Error> {
        let reader: BufReader<File> = self.create_reader("LINIE")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut linies: Vec<Line> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let field: &str = &line[8..9];

            if field == "N" {
                let line_n: RawLinieN = RawLinieN::from_line(&line);
                let mut line_f: Option<RawLinieF> = None;
                let mut line_b: Option<RawLinieB> = None;

                while let Some(Ok(line2)) = lines.next() {
                    let field: &str = &line2[8..9];

                    if field == "F" {
                        line_f = Some(RawLinieF::from_line(&line2));
                    } else if field == "B" {
                        line_b = Some(RawLinieB::from_line(&line2));
                    } else {
                        break;
                    }
                }

                if line_f.is_none() || line_b.is_none() {
                    continue;
                }

                let linie: Line = Line {
                    id: line_n.number,
                    name: line_n.name,
                    color_type: line_f.unwrap().color_type,
                    color: line_b.unwrap().color,
                };

                linies.push(linie);
            }
        }

        return Ok(linies);
    }

    pub fn get_fahrplans(&self) -> Result<Vec<Fahrplan>, Error> {
        let reader: BufReader<File> = self.create_reader("FPLAN")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut fplans: Vec<Fahrplan> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            if line.starts_with("*Z") {
                let line_z: RawFahrplanZ = RawFahrplanZ::from_line(&line);

                if line_z.agency_id != self.agency_id {
                    continue;
                }

                let mut line_g: Option<RawFahrplanG> = None;
                let mut line_a: Option<RawFahrplanA> = None;
                let mut line_l: Option<RawFahrplanL> = None;
                let mut line_r: Option<RawFahrplanR> = None;
                let mut stops: Vec<RawFahrplanStop> = Vec::new();

                while let Some(Ok(line2)) = lines.next() {
                    if line2.starts_with("*G") {
                        line_g = Some(RawFahrplanG::from_line(&line2));
                    } else if line2.starts_with("*A VE") {
                        line_a = Some(RawFahrplanA::from_line(&line2));
                    } else if line2.starts_with("*L") {
                        line_l = Some(RawFahrplanL::from_line(&line2));
                    } else if line2.starts_with("*R") {
                        line_r = Some(RawFahrplanR::from_line(&line2));
                    } else if !line2.starts_with("*") {
                        stops.push(RawFahrplanStop::from_line(&line2));
                    } else {
                        break;
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

                fplans.push(fplan);
            }
        }

        return Ok(fplans);
    }
}
