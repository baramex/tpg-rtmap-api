use chrono::{NaiveDate, NaiveTime};
use unicode_segmentation::UnicodeSegmentation;

use crate::model::{
    bitfield::Bitfield,
    direction::Direction as RouteDirection,
    direction_leg::{self, DirectionLeg},
    leg_step::LegStep,
    line::{Line, TransportMode},
    shape::Shape,
    shape_stop::ShapeStop,
    stop::Stop,
    trip::Trip,
    trip_stop::{self, TripStop},
    types::{ColorType, Direction},
};
use std::{
    cmp,
    fs::File,
    io::{BufRead, BufReader, Error, Lines},
    panic,
    path::PathBuf,
};

use super::maps::Maps;

pub struct HRDF {
    pub directory: PathBuf,
    pub agency_id: String,
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
            pub fn from_line(line: &str) -> Result<Self, Box<dyn std::error::Error>> {
                $(
                    let chars: Vec<&str> = UnicodeSegmentation::graphemes(line, true).collect();
                    let str = chars[$start..cmp::min($end, chars.len())].join("");
                    let $field_name = str.trim().parse::<$field_type>();

                    if($field_name.is_err()) {
                        Err(format!("Failed to parse field {} from {}", stringify!($field_name), str))?;
                    }
                )*
                Ok($record_name {
                    $(
                        $field_name: $field_name.unwrap(),
                    )*
                })
            }
        }
    }
}

define_record! {
    RawFahrplanZ {
        journey_number: i32 => 3 => 9,
        agency_id: String => 10 => 16,
        option_count: i16 => 19 => 22,
    }
}

define_record! {
    RawFahrplanG {
        transport_mode: TransportMode => 3 => 6,
        origin_id: i32 => 7 => 14,
        destination_id: i32 => 15 => 22,
    }
}

define_record! {
    RawFahrplanA {
        _origin_id: i32 => 6 => 13,
        _destination_id: i32 => 14 => 21,
        bit_field_number: i32 => 22 => 28,
    }
}

define_record! {
    RawFahrplanL {
        line_number: i32 => 4 => 11,
        _origin_id: i32 => 12 => 19,
        _destination_id: i32 => 20 => 27,
    }
}

define_record! {
    RawFahrplanR {
        direction: Direction => 3 => 4,
        _direction_number: i32 => 6 => 12,
        _origin_id: i32 => 13 => 20,
        _destination_id: i32 => 21 => 28,
    }
}

define_record! {
    RawFahrplanStop {
        id: i32 => 0 => 7,
        _name: String => 8 => 28,
        arrival_time: String => 30 => 35,
        departure_time: String => 37 => 42,
    }
}

define_record! {
    RawLinieN {
        number: i32 => 0 => 7,
        name: String => 12 => 22,
    }
}

define_record! {
    RawLinieF {
        _number: i32 => 0 => 7,
        color_type: ColorType => 10 => 21,
    }
}

define_record! {
    RawLinieB {
        _number: i32 => 0 => 7,
        color: String => 10 => 21,
    }
}

define_record! {
    RawBitfeld {
        number: i32 => 0 => 6,
        days: String => 7 => 99,
    }
}

define_record! {
    RawStop {
        id: i32 => 0 => 7,
        lon: f64 => 10 => 18,
        lat: f64 => 20 => 29,
        name: String => 39 => 90,
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

pub struct CornerDates {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
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

    pub fn extract_bitfield_ids(&self, fahrplans: &Vec<Fahrplan>) -> Vec<i32> {
        let mut bitfield_ids: Vec<i32> = Vec::new();

        for fahrplan in fahrplans {
            if !bitfield_ids.contains(&fahrplan.a.bit_field_number) {
                bitfield_ids.push(fahrplan.a.bit_field_number);
            }
        }

        return bitfield_ids;
    }

    pub fn retrieve_bitfields(&self, ids: Vec<i32>) -> Result<Vec<Bitfield>, Error> {
        let reader: BufReader<File> = self.create_reader("BITFELD")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut bitfields: Vec<Bitfield> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let bf_line: RawBitfeld = RawBitfeld::from_line(&line).unwrap();

            if ids.contains(&bf_line.number) {
                let bitfield: Bitfield = Bitfield {
                    id: bf_line.number,
                    days: Bitfield::convert_hex_to_bits(bf_line.days.as_str()),
                };

                bitfields.push(bitfield);
            }
        }

        return Ok(bitfields);
    }

    pub fn get_corner_dates(&self) -> Result<CornerDates, Error> {
        let reader: BufReader<File> = self.create_reader("ECKDATEN")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let corner_dates = CornerDates {
            start_date: NaiveDate::parse_from_str(lines.next().unwrap()?.as_str(), "%d.%m.%Y")
                .unwrap(),
            end_date: NaiveDate::parse_from_str(lines.next().unwrap()?.as_str(), "%d.%m.%Y")
                .unwrap(),
        };

        return Ok(corner_dates);
    }

    pub fn get_lines(&self) -> Result<Vec<Line>, Error> {
        let reader: BufReader<File> = self.create_reader("LINIE")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut linies: Vec<Line> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let field: &str = &line[8..9];

            if field == "N" {
                let line_n: RawLinieN = RawLinieN::from_line(&line).unwrap();
                let mut line_f: Option<RawLinieF> = None;
                let mut line_b: Option<RawLinieB> = None;

                while let Some(Ok(line2)) = lines.next() {
                    let field: &str = &line2[8..9];

                    if field == "F" {
                        line_f = Some(RawLinieF::from_line(&line2).unwrap());
                    } else if field == "B" {
                        line_b = Some(RawLinieB::from_line(&line2).unwrap());
                    } else {
                        break;
                    }
                }

                let linie: Line = Line {
                    id: line_n.number,
                    name: line_n.name,
                    color_type: if line_f.is_none() {
                        ColorType::Unknown
                    } else {
                        line_f.unwrap().color_type
                    },
                    color: if line_b.is_none() {
                        String::new()
                    } else {
                        line_b.unwrap().color
                    },
                };

                linies.push(linie);
            }
        }

        return Ok(linies);
    }

    pub fn retrieve_stops(&self, ids: Vec<i32>) -> Result<Vec<Stop>, Error> {
        let reader: BufReader<File> = self.create_reader("BFKOORD_WGS")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut stops: Vec<Stop> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let stop: RawStop = RawStop::from_line(&line).unwrap();

            if ids.contains(&stop.id) {
                let stop: Stop = Stop {
                    id: stop.id,
                    name: stop.name,
                    latitude: stop.lat,
                    longitude: stop.lon,
                };

                stops.push(stop);
            }
        }

        return Ok(stops);
    }

    pub fn extract_stop_ids(&self, fahrplans: &Vec<Fahrplan>) -> Vec<i32> {
        let mut stop_ids: Vec<i32> = Vec::new();

        for fahrplan in fahrplans {
            for stop in &fahrplan.stops {
                if !stop_ids.contains(&stop.id) {
                    stop_ids.push(stop.id);
                }
            }
        }

        return stop_ids;
    }

    pub fn to_trips_and_shapes_and_shape_stops(
        &self,
        fahrplans: &Vec<Fahrplan>,
    ) -> (Vec<Trip>, Vec<Shape>, Vec<ShapeStop>) {
        let mut trips: Vec<Trip> = Vec::new();
        let mut i: i32 = 1;

        let mut shapes: Vec<Shape> = Vec::new();
        let mut si: i32 = 0;

        let mut shape_stops: Vec<ShapeStop> = Vec::new();
        let mut ssi: i32 = 1;

        for fahrplan in fahrplans {
            let mut j: i16 = 0;

            let identifier = fahrplan
                .stops
                .iter()
                .map(|stop| {
                    j += 1;
                    j.to_string() + stop.id.to_string().as_str()
                })
                .collect::<Vec<String>>()
                .join("");

            let mut shape: Option<&Shape> =
                shapes.iter().find(|shape| shape.identifier == identifier);
            if shape.is_none() {
                si += 1;
                let temp_shape = Shape { id: si, identifier };

                let mut y = 1;
                for stop in &fahrplan.stops {
                    let shape_stop: ShapeStop = ShapeStop {
                        id: ssi,
                        shape_id: si,
                        stop_id: stop.id,
                        sequence: y,
                    };

                    shape_stops.push(shape_stop);
                    y += 1;
                    ssi += 1;
                }

                shapes.push(temp_shape);
                shape = Some(shapes.last().unwrap());
            }

            let trip: Trip = Trip {
                id: i,
                journey_number: fahrplan.z.journey_number,
                option_count: fahrplan.z.option_count,
                shape_id: Some(shape.unwrap().id),
                direction_id: None,
                transport_mode: fahrplan.g.transport_mode,
                origin_id: fahrplan.g.origin_id,
                destination_id: fahrplan.g.destination_id,
                bitfield_id: fahrplan.a.bit_field_number,
                line_id: fahrplan.l.line_number,
                direction: fahrplan.r.direction,
                arrival_time: NaiveTime::from_hms_opt(
                    fahrplan.stops[fahrplan.stops.len() - 1].arrival_time[1..3]
                        .parse::<u32>()
                        .unwrap()
                        % 24, // TODO: manage trips that are after 00h
                    fahrplan.stops[fahrplan.stops.len() - 1].arrival_time[3..5]
                        .parse()
                        .unwrap(),
                    0,
                )
                .unwrap(),
                departure_time: NaiveTime::from_hms_opt(
                    fahrplan.stops[0].departure_time[1..3]
                        .parse::<u32>()
                        .unwrap()
                        % 24, // TODO: manage trips that are after 00h
                    fahrplan.stops[0].departure_time[3..5].parse().unwrap(),
                    0,
                )
                .unwrap(),
            };

            trips.push(trip);
            i += 1;
        }

        return (trips, shapes, shape_stops);
    }

    pub fn to_trips_and_directions(
        &self,
        fahrplans: &Vec<Fahrplan>,
    ) -> (Vec<Trip>, Vec<RouteDirection>) {
        let mut trips: Vec<Trip> = Vec::new();
        let mut i: i32 = 1;

        let mut directions: Vec<RouteDirection> = Vec::new();
        let mut si: i32 = 0;

        for fahrplan in fahrplans {
            let mut j: i16 = 0;

            let identifier = fahrplan
                .stops
                .iter()
                .map(|stop| {
                    j += 1;
                    j.to_string() + stop.id.to_string().as_str()
                })
                .collect::<Vec<String>>()
                .join("");

            let mut direction: Option<&RouteDirection> = directions
                .iter()
                .find(|direction| direction.identifier == identifier);

            if direction.is_none() {
                si += 1;
                let temp_dir = RouteDirection {
                    id: si,
                    identifier,
                    origin_id: fahrplan.g.origin_id,
                    destination_id: fahrplan.g.destination_id,
                };

                directions.push(temp_dir);
                direction = Some(directions.last().unwrap());
            }

            let trip: Trip = Trip {
                id: i,
                journey_number: fahrplan.z.journey_number,
                option_count: fahrplan.z.option_count,
                shape_id: None,
                direction_id: Some(direction.unwrap().id),
                transport_mode: fahrplan.g.transport_mode,
                origin_id: fahrplan.g.origin_id,
                destination_id: fahrplan.g.destination_id,
                bitfield_id: fahrplan.a.bit_field_number,
                line_id: fahrplan.l.line_number,
                direction: fahrplan.r.direction,
                arrival_time: NaiveTime::from_hms_opt(
                    fahrplan.stops[fahrplan.stops.len() - 1].arrival_time[1..3]
                        .parse::<u32>()
                        .unwrap()
                        % 24, // TODO: manage trips that are after 00h
                    fahrplan.stops[fahrplan.stops.len() - 1].arrival_time[3..5]
                        .parse()
                        .unwrap(),
                    0,
                )
                .unwrap(),
                departure_time: NaiveTime::from_hms_opt(
                    fahrplan.stops[0].departure_time[1..3]
                        .parse::<u32>()
                        .unwrap()
                        % 24, // TODO: manage trips that are after 00h
                    fahrplan.stops[0].departure_time[3..5].parse().unwrap(),
                    0,
                )
                .unwrap(),
            };

            trips.push(trip);
            i += 1;
        }

        return (trips, directions);
    }

    // create trip stops taking account of the route duration (from google maps)
    pub async fn get_trip_stops_with_directions(
        &self,
        fahrplans: &Vec<Fahrplan>,
        directions: &Vec<RouteDirection>,
        stops: &Vec<Stop>,
        maps: Maps,
    ) -> Result<(Vec<TripStop>, Vec<DirectionLeg>, Vec<LegStep>), Error> {
        let mut trip_stops: Vec<TripStop> = Vec::new();
        let mut a: i32 = 1;
        let mut i: i32 = 1;

        let mut direction_legs: Vec<DirectionLeg> = Vec::new();
        let mut leg_steps: Vec<LegStep> = Vec::new();
        let mut leg_id = 1;
        let mut step_id = 1;

        for fahrplan in fahrplans {
            let mut j: i16 = 0;

            let identifier = fahrplan
                .stops
                .iter()
                .map(|stop| {
                    j += 1;
                    j.to_string() + stop.id.to_string().as_str()
                })
                .collect::<Vec<String>>()
                .join("");
            let direction = directions.iter().find(|d| d.identifier == identifier);

            if direction.is_none() {
                continue;
            }

            let tstops: Vec<&Stop> = fahrplan
                .stops
                .iter()
                .map(|stop| stops.iter().find(|s| s.id == stop.id).unwrap())
                .collect();

            let mut dlegs: Vec<&DirectionLeg> = direction_legs
                .iter()
                .filter(|dleg| dleg.direction_id == direction.unwrap().id)
                .collect::<Vec<&DirectionLeg>>();

            if dlegs.len() == 0 {
                let (dl, ls) = maps
                    .get_direction_sub_from_direction(
                        &direction.unwrap(),
                        &tstops,
                        stops,
                        leg_id,
                        step_id,
                    )
                    .await
                    .unwrap();

                direction_legs.extend(dl);
                leg_steps.extend(ls);

                dlegs = direction_legs
                    .iter()
                    .filter(|dleg| dleg.direction_id == direction.unwrap().id)
                    .collect::<Vec<&DirectionLeg>>();

                leg_id = direction_legs.len() as i32 + 1;
                step_id = leg_steps.len() as i32 + 1;
            }

            let mut h: i16 = 1;
            for stop in &fahrplan.stops {
                let previous_stop: Option<i32> = if trip_stops.last().is_some() {
                    Some(trip_stops.last().unwrap().stop_id)
                } else {
                    None
                };
                let previous_departure: Option<NaiveTime> = if trip_stops.last().is_some() {
                    Some(trip_stops.last().unwrap().departure_time.unwrap())
                } else {
                    None
                };
                let mut arrival_time: Option<NaiveTime> = if stop.arrival_time.is_empty() {
                    None
                } else {
                    Some(
                        NaiveTime::from_hms_opt(
                            stop.arrival_time[1..3].parse::<u32>().unwrap() % 24, // TODO: manage trips that are after 00h
                            stop.arrival_time[3..5].parse().unwrap(),
                            0,
                        )
                        .unwrap(),
                    )
                };

                let departure_time: Option<NaiveTime> = if stop.departure_time.is_empty() {
                    None
                } else {
                    Some(
                        NaiveTime::from_hms_opt(
                            stop.departure_time[1..3].parse::<u32>().unwrap() % 24, // TODO: manage trips that are after 00h
                            stop.departure_time[3..5].parse().unwrap(),
                            0,
                        )
                        .unwrap(),
                    )
                };

                let stop_duration: i64 = if arrival_time.is_some() && departure_time.is_some() {
                    departure_time
                        .unwrap()
                        .signed_duration_since(arrival_time.unwrap())
                        .num_seconds()
                        + 15
                } else {
                    0
                };

                if arrival_time.is_some() && previous_departure.is_some() && previous_stop.is_some()
                {
                    let trip_duration = dlegs
                        .iter()
                        .find(|dleg| {
                            dleg.origin_id == previous_stop.unwrap()
                                && dleg.destination_id == stop.id
                        })
                        .unwrap()
                        .duration;
                    let real_arrival_time = previous_departure.unwrap()
                        + chrono::Duration::seconds(trip_duration as i64);

                    let difference = arrival_time
                        .unwrap()
                        .signed_duration_since(real_arrival_time);
                    if difference.num_seconds().abs() < 60 {
                        arrival_time = Some(real_arrival_time);
                    }
                }

                let trip_stop: TripStop = TripStop {
                    id: i,
                    stop_id: stop.id,
                    trip_id: a,
                    sequence: h,
                    arrival_time,
                    departure_time: if arrival_time.is_some() {
                        Some(arrival_time.unwrap() + chrono::Duration::seconds(stop_duration))
                    } else {
                        departure_time
                    },
                };

                trip_stops.push(trip_stop);

                h += 1;
                i += 1;
            }

            a += 1;
        }

        Ok((trip_stops, direction_legs, leg_steps))
    }

    pub fn to_trip_stops(&self, fahrplans: &Vec<Fahrplan>) -> Vec<TripStop> {
        let mut trip_stops: Vec<TripStop> = Vec::new();
        let mut i: i32 = 1;
        let mut a: i32 = 1;

        for fahrplan in fahrplans {
            let mut j: i16 = 1;

            for stop in &fahrplan.stops {
                let trip_stop: TripStop = TripStop {
                    id: a,
                    stop_id: stop.id,
                    trip_id: i,
                    sequence: j,
                    arrival_time: if stop.arrival_time.is_empty() {
                        None
                    } else {
                        Some(
                            NaiveTime::from_hms_opt(
                                stop.arrival_time[1..3].parse::<u32>().unwrap() % 24, // TODO: manage trips that are after 00h
                                stop.arrival_time[3..5].parse().unwrap(),
                                0,
                            )
                            .unwrap(),
                        )
                    },
                    departure_time: if stop.departure_time.is_empty() {
                        None
                    } else {
                        Some(
                            NaiveTime::from_hms_opt(
                                stop.departure_time[1..3].parse::<u32>().unwrap() % 24, // TODO: manage trips that are after 00h
                                stop.departure_time[3..5].parse().unwrap(),
                                15,
                            )
                            .unwrap(),
                        )
                    },
                };

                trip_stops.push(trip_stop);
                j += 1;
                a += 1;
            }

            i += 1;
        }

        return trip_stops;
    }

    pub fn get_fahrplans(&self) -> Result<Vec<Fahrplan>, Error> {
        let reader: BufReader<File> = self.create_reader("FPLAN")?;
        let mut lines: Lines<BufReader<File>> = reader.lines();

        let mut fplans: Vec<Fahrplan> = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            if line.starts_with("*Z") {
                let line_z: RawFahrplanZ = RawFahrplanZ::from_line(&line).unwrap();

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
                        line_g = Some(RawFahrplanG::from_line(&line2).unwrap());
                    } else if line2.starts_with("*A VE") {
                        line_a = Some(RawFahrplanA::from_line(&line2).unwrap_or(RawFahrplanA {
                            _origin_id: 0,
                            _destination_id: 0,
                            bit_field_number: 17
                        }));
                    } else if line2.starts_with("*L") {
                        line_l = Some(RawFahrplanL::from_line(&line2).unwrap());
                    } else if line2.starts_with("*R") {
                        line_r = Some(RawFahrplanR::from_line(&line2).unwrap());
                    } else if !line2.starts_with("*") {
                        stops.push(RawFahrplanStop::from_line(&line2).unwrap());
                    } else if !line2.starts_with("*A NF") && !line2.starts_with("*A SM") && !line2.starts_with("*A SD") {
                        break;
                    }
                }

                if line_g.is_none() || line_a.is_none() || line_l.is_none() || line_r.is_none() {
                    println!("Incomplete fahrplan: {:?} {:?} {:?} {:?} {:?}", line_z.journey_number, line_g, line_a, line_l, line_r);
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
