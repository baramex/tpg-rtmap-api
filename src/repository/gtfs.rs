use crate::model::{
    line::{Line, LineDescription, TransportMode},
    stop::Stop,
};
use derive_more::Display;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};

pub struct GTFS {
    pub directory: PathBuf,
    pub agency_id: String,
}

#[derive(Deserialize)]
struct RawRoute {
    route_id: String,
    agency_id: String,
    route_short_name: String,
    route_long_name: String,
    route_desc: LineDescription,
    route_type: String,
}

#[derive(Deserialize)]
struct RawStop {
    stop_id: String,
    stop_name: String,
    stop_lat: f64,
    stop_lon: f64,
    location_type: String,
    parent_station: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawHaltestellen {
    FP_ID: u16,                       // date,
    TU_CODE: String,                  // company
    TU_BEZEICHNUNG: String,           // company name
    TU_ABKUERZUNG: String,            // company short name
    FARTNUMMER: String,               // journey id
    BPUIC: String,                    // stop id (UIC)
    BP_BEZEICHNUNG: String,           // stop name
    BP_ABKUERZUNG: String,            // stop short name
    KANTON: String,                   // canton
    SLOID: String,                    // stop id (Swiss loc ID)
    VM_ART: TransportMode,            // transport mode
    FAHRTAGE: String,                 // days of operation
    AB_ZEIT_KB: String,               // departure time
    AN_ZEIT_KB: String,               // arrival time
    RICHTUNG_TEXT_AGGREGIERT: String, // direction
    END_BP_BEZEICHNUNG: String,       // destination
    LINIE: String,                    // line number
    BP_ID: String,                    // stop id (internal ID)
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

#[derive(Debug, Display)]
pub enum ReadError {
    UnableToRead,
    UnableToParse,
}

impl GTFS {
    fn read_file<T: for<'a> Deserialize<'a>>(&self, file_name: &str) -> Result<Vec<T>, ReadError> {
        let path: PathBuf = self.directory.join(file_name);
        let file_res: Result<File, std::io::Error> = File::open(path);
        if file_res.is_err() {
            return Err(ReadError::UnableToRead);
        }
        let file: File = file_res.unwrap();
        let reader: BufReader<File> = BufReader::new(file);

        let mut binding = csv::Reader::from_reader(reader);
        let records = binding.deserialize::<T>();
        let result: Result<Vec<T>, csv::Error> = records.collect();

        match result {
            Ok(result) => Ok(result),
            Err(_) => Err(ReadError::UnableToParse),
        }
    }

    /*pub fn read_lines(&self, modes: Vec<TransportMode>) -> Result<Vec<Line>, ReadError> {
        let routes: Vec<RawRoute> = self.read_file::<RawRoute>("routes.txt")?;

        let mut lines: Vec<Line> = Vec::new();

        for route in routes {
            let mode: TransportMode = TransportMode::from_description(&route.route_desc);
            if route.agency_id != self.agency_id || !modes.contains(&mode) {
                continue;
            }
            lines.push(Line::new(
                route.route_id,
                route.route_short_name,
                String::new(),
                route.route_desc,
                mode,
            ));
        }

        Ok(lines)
    }*/

    pub fn read_haltestellen(
        &self,
        modes: Vec<TransportMode>,
    ) -> Result<Vec<RawHaltestellen>, ReadError> {
        let mut raw: Vec<RawHaltestellen> =
            self.read_file::<RawHaltestellen>("haltestellen_2023.csv")?;

        raw = raw
            .iter()
            .filter_map(|a: &RawHaltestellen| {
                if a.TU_CODE == self.agency_id && modes.contains(&a.VM_ART) {
                    return Some(a.clone());
                } else {
                    return None;
                }
            })
            .collect();

        Ok(raw)
    }

    /*pub fn get_stops_from_haltestellen(
        &self,
        haltestellen: Vec<RawHaltestellen>,
        all_stops: &Vec<Stop>,
    ) -> Vec<Stop> {
        let mut ids: Vec<String> = Vec::new();
        let mut stops: Vec<Stop> = Vec::new();

        for haltestelle in haltestellen {
            let id: String = haltestelle.BPUIC;
            if !ids.contains(&id) {
                let stop_res: Option<Stop> = self.get_stop_in_stops(id.as_str(), all_stops);
                ids.push(id);
                if stop_res.is_none() {
                    continue;
                }
                stops.push(stop_res.unwrap());
            }
        }

        stops
    }*/

    pub fn read_all_stops(&self) -> Result<Vec<Stop>, ReadError> {
        let raw_stops: Vec<RawStop> = self.read_file::<RawStop>("stops.txt")?;

        let mut stops: Vec<Stop> = Vec::new();

        for stop in raw_stops {
            stops.push(Stop::new(
                stop.stop_id,
                stop.stop_lat,
                stop.stop_lon,
                stop.stop_name,
            ));
        }

        Ok(stops)
    }

    /*pub fn get_stop_in_stops(&self, id: &str, stops: &Vec<Stop>) -> Option<Stop> {
        for stop in stops {
            if stop.reference.eq(id) {
                return Some(stop.clone());
            }
        }

        None
    }*/
}
