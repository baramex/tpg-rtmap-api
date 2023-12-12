mod api;
mod model;
mod repository;

use crate::repository::database::Table;

use std::{env, path::Path, str::FromStr};

use api::{
    direction::{get_direction, get_direction_leg_steps, get_direction_legs},
    leg::{get_leg, get_leg_steps},
    line::{get_line, get_lines},
    shape::{get_shape, get_shape_points, get_shape_stops},
    stop::{get_stop, get_stops},
    trip::{get_trip, get_trip_stops, get_trips},
};

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use model::{
    bitfield::Bitfield, direction::Direction, direction_leg::DirectionLeg,
    information::Information, leg_step::LegStep, line::Line, shape::Shape, shape_point::ShapePoint,
    shape_stop::ShapeStop, stop::Stop, trip::Trip, trip_stop::TripStop,
};
use repository::{
    database::Database,
    hrdf::{CornerDates, Fahrplan, HRDF},
    maps::Maps,
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // init database
    let database: Database = Database::init(
        PgPoolOptions::new(),
        PgConnectOptions::from_str(env::var("DATABASE_URL").unwrap().as_str())
            .unwrap()
            .disable_statement_logging(),
    )
    .await
    .unwrap();

    // retreive info from gtfs
    /*let gtfs: GTFS = GTFS {
        directory: Path::new("/Users/baramex/Desktop/tpg-rtmap-api/src/gtfs").to_path_buf(),
        agency_id: env::var("AGENCY_ID").unwrap(),
    };*/
    /*let lines: Vec<Line> = gtfs
        .read_lines(vec![TransportMode::Bus, TransportMode::Tramway])
        .unwrap();
    for line in lines {
        println!(
            "Id: {}, ref: {}, name: {}, mode: {:?}",
            line.id, line.reference, line.name, line.mode
        );
    }*/
    /*let all_stops: Vec<Stop> = gtfs.read_all_stops().unwrap();
    let haltestellen: Vec<RawHaltestellen> = gtfs
        .read_haltestellen(vec![TransportMode::Bus, TransportMode::Tramway])
        .unwrap();
    let stops: Vec<Stop> = gtfs.get_stops_from_haltestellen(haltestellen, &all_stops);
    println!("{:#?}", stops);*/

    // init database: create tables
    let _ = Bitfield::create_table(&database).await;
    let _ = Line::create_table(&database).await;
    let _ = Stop::create_table(&database).await;
    let _ = Shape::create_table(&database).await;
    let _ = Trip::create_table(&database).await;
    let _ = TripStop::create_table(&database).await;
    let _ = Information::create_table(&database).await;
    let _ = ShapeStop::create_table(&database).await;
    let _ = ShapePoint::create_table(&database).await;
    let _ = Direction::create_table(&database).await;
    let _ = DirectionLeg::create_table(&database).await;
    let _ = LegStep::create_table(&database).await;

    // retrieve data from hrdf and insert into database
    let hrdf: HRDF = HRDF {
        directory: Path::new(&env::var("HRDF_PATH").unwrap().parse::<String>().unwrap())
            .to_path_buf(),
        agency_id: env::var("AGENCY_ID").unwrap().parse::<String>().unwrap(),
    };

    let maps = Maps {
        api_key: env::var("MAPS_API_KEY").unwrap().parse::<String>().unwrap(),
    };

    let insert_bitfields = false;
    let insert_lines = false;
    let insert_stops = false;

    let insert_trips = false;
    let insert_trip_stops = false;

    let insert_directions = false;

    let insert_shapes = false; // UNSTABLE
    let insert_shape_points = false; // UNSTABLE

    let insert_information = false;

    let mut fahrplans: Vec<Fahrplan> = Vec::new();
    let mut stops: Vec<Stop> = Vec::new();

    if insert_lines {
        println!("Getting lines...");
        let lines: Vec<Line> = hrdf.get_lines().unwrap();
        println!("Got lines: {}", lines.len());

        println!("Inserting lines...");
        let _l = Database::insert_many::<Line>(&database, &lines).await;
        println!("Inserted lines");
    }

    if insert_information {
        println!("Getting information...");
        let corner_dates: CornerDates = hrdf.get_corner_dates().unwrap();
        println!("Got corner dates");

        println!("Inserting information...");
        let _i = Database::insert_many::<Information>(
            &database,
            &vec![Information {
                id: 1,
                start_date: corner_dates.start_date,
                end_date: corner_dates.end_date,
            }],
        )
        .await;
        println!("Inserted information");
    }

    if insert_trips
        || insert_trip_stops
        || insert_stops
        || insert_bitfields
        || insert_shapes
        || insert_directions
    {
        println!("Getting fahrplans...");
        let res = hrdf.get_fahrplans();
        if res.is_err() {
            panic!("Error: {:?}", res.err().unwrap());
        }
        fahrplans = res.unwrap();
        println!("Got fahrplans: {}", fahrplans.len());
    }

    if insert_bitfields {
        println!("Getting bitfields...");
        let bitfield_ids: Vec<i32> = hrdf.extract_bitfield_ids(&fahrplans);
        let bitfields: Vec<Bitfield> = hrdf.retrieve_bitfields(bitfield_ids).unwrap();
        println!("Got bitfields: {}", bitfields.len());

        println!("Inserting bitfields...");
        let _b = Database::insert_many::<Bitfield>(&database, &bitfields).await;
        println!("Inserted bitfields");
    }

    if insert_stops || insert_shape_points || insert_directions {
        println!("Getting stops...");
        let stops_id: Vec<i32> = hrdf.extract_stop_ids(&fahrplans);
        stops = hrdf.retrieve_stops(stops_id).unwrap();
        println!("Got stops: {}", stops.len());

        println!("Inserting stops...");
        let _s = Database::insert_many::<Stop>(&database, &stops).await;
        println!("Inserted stops");
    }

    if insert_trip_stops && !insert_directions {
        println!("Getting trip stops...");
        let trip_stops = hrdf.to_trip_stops(&fahrplans);
        println!("Got trip stops: {}", trip_stops.len());

        println!("Inserting trip stops...");
        let _ts = Database::insert_many::<TripStop>(&database, &trip_stops).await;
        println!("Inserted trip stops");
    }

    if insert_trips && (insert_shapes || insert_shape_points) { // UNSTABLE
        println!("Getting trips and shapes...");
        let result = hrdf.to_trips_and_shapes_and_shape_stops(&fahrplans);
        let trips: Vec<Trip> = result.0;
        let shapes: Vec<Shape> = result.1;
        let shape_stops: Vec<ShapeStop> = result.2;
        println!("Got trips: {}", trips.len());
        println!("Got shapes: {}", shapes.len());
        println!("Got shape stops: {}", shape_stops.len());

        if insert_shapes {
            println!("Inserting shapes...");
            let _s = Database::insert_many::<Shape>(&database, &shapes).await;
            println!("Inserted shapes");

            println!("Inserting shape stops...");
            let _ss = Database::insert_many::<ShapeStop>(&database, &shape_stops).await;
            println!("Inserted shape stops");
        }
        if insert_trips {
            println!("Inserting trips...");
            let _t = Database::insert_many::<Trip>(&database, &trips).await;
            println!("Inserted trips");
        }
        if insert_shape_points {
            // TO REDO (use maps.get_shape_points_from_shape_stops)
            /*println!("Getting shape points...");
            let shape_points: Vec<ShapePoint> =
                hrdf.fetch_shape_points(&shapes, &shape_stops, &stops).await;
            println!("Got shape points: {}", shape_points.len());

            println!("Inserting shape points...");
            let _sp = Database::insert_many::<ShapePoint>(&database, &shape_points).await;
            println!("Inserted shape points");*/
        }
    } else if insert_trips || insert_directions { // STABLE
        println!("Getting trips and directions...");
        let result = hrdf.to_trips_and_directions(&fahrplans);
        let trips: Vec<Trip> = result.0;
        let directions: Vec<Direction> = result.1;
        println!("Got trips: {}", trips.len());
        println!("Got directions: {}", directions.len());

        let mut trip_stops: Vec<TripStop> = Vec::new();	
        if insert_directions {
            println!("Getting trip stops, direction legs and steps...");
            let (_trip_stops, direction_legs, leg_steps) = hrdf
                .get_trip_stops_with_directions(&fahrplans, &directions, &stops, maps)
                .await
                .unwrap();

            trip_stops = _trip_stops;
            println!("Got trip stops: {}", trip_stops.len());
            println!("Got direction legs: {}", direction_legs.len());
            println!("Got leg steps: {}", leg_steps.len());

            println!("Inserting direction, steps and legs...");
            let _d = Database::insert_many::<Direction>(&database, &directions).await;
            let _dl = Database::insert_many::<DirectionLeg>(&database, &direction_legs).await;
            let _ls = Database::insert_many::<LegStep>(&database, &leg_steps).await;
            println!("Inserted direction, steps and legs");
        }

        if insert_trips {
            println!("Inserting trips...");
            let _t = Database::insert_many::<Trip>(&database, &trips).await;
            println!("Inserted trips");
        }

        if insert_trip_stops {
            println!("Inserting trip stops...");
            let _ts = Database::insert_many::<TripStop>(&database, &trip_stops).await;
            println!("Inserted trip stops");
        }
    }

    // init http server
    HttpServer::new(move || {
        let db_data: Data<Database> = Data::new(database.clone());

        let logger: Logger = Logger::default();

        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET"]);

        App::new()
            .app_data(db_data)
            .wrap(cors)
            .wrap(logger)
            .service(get_line)
            .service(get_lines)
            .service(get_stop)
            .service(get_stops)
            .service(get_trip)
            .service(get_trip_stops)
            .service(get_trips)
            .service(get_shape)
            .service(get_shape_points)
            .service(get_shape_stops)
            .service(get_direction)
            .service(get_direction_legs)
            .service(get_direction_leg_steps)
            .service(get_leg)
            .service(get_leg_steps)
    })
    .bind(("127.0.0.1", 10000))?
    .run()
    .await
}
