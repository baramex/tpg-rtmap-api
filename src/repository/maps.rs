use reqwest::{Error, Response};
use serde::{Serialize, Deserialize};

use crate::model::{
    direction::Direction, direction_leg::DirectionLeg, leg_step::LegStep,
    stop::Stop, trip::Trip, trip_stop::TripStop,
};

#[derive(Debug, Deserialize, Serialize)]
struct DirectionResponse {
    routes: Vec<RouteResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RouteResponse {
    bounds: Bounds,
    legs: Vec<LegResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LegResponse {
    distance: Distance,
    duration: Duration,
    start_location: Position,
    end_location: Position,
    steps: Vec<StepResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StepResponse {
    distance: Distance,
    duration: Duration,
    start_location: Position,
    end_location: Position,
}

#[derive(Debug, Deserialize, Serialize)]
struct Distance {
    text: String,
    value: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Duration {
    text: String,
    value: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Bounds {
    northeast: Position,
    southwest: Position,
}

#[derive(Debug, Deserialize, Serialize)]
struct Position {
    lat: f64,
    lng: f64,
}

pub struct Maps {
    api_key: String,
}

impl Maps {
    pub async fn get_direction_from_trip(
        &self,
        trip: &Trip,
        _trip_stops: &Vec<TripStop>,
        stops: &Vec<Stop>,
        dir_id: i32,
        leg_id: i32,
        step_id: i32,
        path_id: i32,
    ) -> Result<(Direction, Vec<DirectionLeg>, Vec<LegStep>), Error> {
        let mut trip_stops: Vec<TripStop> = _trip_stops.to_vec();
        trip_stops.sort_by_key(|s: &TripStop| s.sequence);

        let direction = Direction {
            id: dir_id,
            identifier: (&trip_stops)
                .into_iter()
                .map(|s| s.sequence.to_string() + s.id.to_string().as_str())
                .collect::<Vec<String>>()
                .join(""),
            origin_id: trip.origin_id,
            destination_id: trip.destination_id,
        };

        let mut direction_legs: Vec<DirectionLeg> = Vec::new();
        let mut leg_steps: Vec<LegStep> = Vec::new();

        let origin = stops
            .iter()
            .find(|s| s.id == trip.origin_id)
            .expect("Origin not found");
        let destination = stops
            .iter()
            .find(|s| s.id == trip.destination_id)
            .expect("Destination not found");

        let waypoints: String = (&trip_stops)[1..trip_stops.len() - 1]
            .iter()
            .map(|trip_stop| {
                let stop = stops.iter().find(|s| s.id == trip_stop.stop_id).unwrap();
                self.format_position(stop.latitude, stop.longitude)
            })
            .collect::<Vec<String>>()
            .join("|");

        let res: Response = reqwest::get(format!(
            "https://maps.googleapis.com/maps/api/directions/json?key={}&origin={}&destination={}&waypoints={}",
            self.api_key,
            self.format_position(origin.latitude, origin.longitude),
            self.format_position(destination.latitude, destination.longitude),
            waypoints
        ))
        .await?;

        if res.status().is_success() {
            let Route: &RouteResponse = res.json::<DirectionResponse>().await?.routes.first().unwrap();
            // TODO
        }

        return Ok((direction, direction_legs, leg_steps));
    }

    fn format_position(&self, lat: f64, lng: f64) -> String {
        return format!("{},{}", lat, lng);
    }
}
