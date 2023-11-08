use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};

use crate::model::{
    direction::Direction, direction_leg::DirectionLeg, leg_step::LegStep, shape_point::ShapePoint,
    shape_stop::ShapeStop, stop::Stop, trip_stop::TripStop,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RoadResponse {
    pub snappedPoints: Vec<SnappedPoint>,
    pub warningMessage: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SnappedPoint {
    pub location: Location,
    pub originalIndex: Option<i32>,
    pub placeId: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

pub struct Maps {
    pub api_key: String,
}

impl Maps {
    pub async fn get_direction_sub_from_direction(
        &self,
        direction: &Direction,
        _trip_stops: &Vec<&TripStop>,
        stops: &Vec<Stop>,
        start_leg_id: i32,
        strat_step_id: i32,
    ) -> Result<(Vec<DirectionLeg>, Vec<LegStep>), Error> {
        let mut trip_stops: Vec<&TripStop> = _trip_stops.to_vec();
        trip_stops.sort_by_key(|s: &&TripStop| s.sequence);

        let mut leg_id = start_leg_id;
        let mut step_id = strat_step_id;

        let mut direction_legs: Vec<DirectionLeg> = Vec::new();
        let mut leg_steps: Vec<LegStep> = Vec::new();

        let mut i: i16 = 1;
        while trip_stops.len() > 0 {
            let max = if trip_stops.len() > 25 {
                25
            } else {
                trip_stops.len()
            };
            let waypoints: Vec<Location> = (&trip_stops)[0..max]
                .iter()
                .map(|trip_stop| {
                    let stop = stops.iter().find(|s| s.id == trip_stop.stop_id).unwrap();
                    Location {
                        latitude: stop.latitude,
                        longitude: stop.longitude,
                    }
                })
                .collect::<Vec<Location>>();

            let res: Response = reqwest::get(format!(
                "https://maps.googleapis.com/maps/api/directions/json?key={}&origin={}&destination={}&waypoints={}",
                self.api_key,
                self.format_position(waypoints[0].latitude, waypoints[0].longitude),
                self.format_position(waypoints.last().unwrap().latitude, waypoints.last().unwrap().longitude),
                if max > 1 { waypoints[1..max-1].iter().map(|waypoint| self.format_position(waypoint.latitude, waypoint.longitude)).collect::<Vec<String>>().join("|") } else { "".to_string() }
            ))
            .await?;

            if res.status().is_success() {
                let direction_response = res.json::<DirectionResponse>().await?;
                let route_response = direction_response.routes.first().unwrap();

                for leg_response in &route_response.legs {
                    let leg = DirectionLeg {
                        id: leg_id,
                        direction_id: direction.id,
                        distance: leg_response.distance.value,
                        duration: leg_response.duration.value,
                        origin_id: trip_stops[i as usize - 1].stop_id,
                        destination_id: trip_stops[i as usize].stop_id,
                        sequence: i,
                    };

                    direction_legs.push(leg);

                    let mut j: i16 = 1;
                    for step in &leg_response.steps {
                        let leg_step = LegStep {
                            id: step_id,
                            distance: step.distance.value,
                            duration: step.duration.value,
                            leg_id,
                            sequence: j,
                            start_lat: step.start_location.lat,
                            start_lng: step.start_location.lng,
                            end_lat: step.end_location.lat,
                            end_lng: step.end_location.lng,
                        };

                        leg_steps.push(leg_step);

                        j += 1;
                        step_id += 1;
                    }

                    i += 1;
                    leg_id += 1;
                }
            }

            trip_stops = (&trip_stops)[max..].to_vec();
        }

        return Ok((direction_legs, leg_steps));
    }

    pub async fn get_shape_points_from_shape_stops(
        &self,
        shape_id: i32,
        shape_stops: &Vec<ShapeStop>,
        stops: &Vec<Stop>,
        start_shape_point_id: i32,
    ) -> Vec<ShapePoint> {
        let mut shape_points: Vec<ShapePoint> = Vec::new();

        let mut shape_point_id = start_shape_point_id;

        let path = shape_stops
            .iter()
            .map(|shape_stop| {
                let stop = stops.iter().find(|s| s.id == shape_stop.stop_id).unwrap();
                self.format_position(stop.latitude, stop.longitude)
            })
            .collect::<Vec<String>>()
            .join("|");

        let res: Response = reqwest::get(format!(
            "https://roads.googleapis.com/v1/snapToRoads?interpolate=true&key={}&path={}",
            "***REMOVED***", path
        ))
        .await
        .unwrap();

        if res.status().is_success() {
            let snapped_points: Vec<SnappedPoint> =
                res.json::<RoadResponse>().await.unwrap().snappedPoints;

            let mut j: i16 = 1;
            for snapped_point in snapped_points {
                let shape_point: ShapePoint = ShapePoint {
                    id: shape_point_id,
                    shape_id,
                    sequence: j,
                    latitude: snapped_point.location.latitude,
                    longitude: snapped_point.location.longitude,
                    shape_stop_id: if snapped_point.originalIndex.is_some() {
                        Some(shape_stops[snapped_point.originalIndex.unwrap() as usize].id)
                    } else {
                        None
                    },
                };

                shape_points.push(shape_point);
                j += 1;
                shape_point_id += 1;
            }
        }

        return shape_points;
    }

    fn format_position(&self, lat: f64, lng: f64) -> String {
        return format!("{},{}", lat, lng);
    }
}
