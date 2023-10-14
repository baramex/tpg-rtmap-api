mod api;
mod model;
mod repository;

use crate::repository::database::Table;

use std::{env, path::Path};

//use api::line::get_line;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use model::{bitfield::Bitfield, line::Line, stop::Stop, trip::Trip, trip_stop::TripStop};
use repository::{
    database::Database,
    hrdf::{Fahrplan, HRDF},
};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // init database
    let database: Database = Database::init(
        PgPoolOptions::new(),
        env::var("DATABASE_URL").unwrap().as_str(),
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
    let _ = Trip::create_table(&database).await;
    let _ = TripStop::create_table(&database).await;

    // retrieve data from hrdf and insert into database
    let hrdf: HRDF = HRDF {
        directory: Path::new(&env::var("HRDF_PATH").unwrap().parse::<String>().unwrap())
            .to_path_buf(),
        agency_id: env::var("AGENCY_ID").unwrap().parse::<String>().unwrap(),
    };

    let insert_bitfields = false;
    let insert_lines = false;
    let insert_stops = false;
    let insert_trips = true;
    let insert_trip_stops = true;

    let mut fahrplans: Vec<Fahrplan> = Vec::new();

    if insert_bitfields {
        println!("Getting bitfields...");
        let bitfields: Vec<Bitfield> = hrdf.get_bitfields().unwrap();
        println!("Got bitfields: {}", bitfields.len());

        println!("Inserting bitfields...");
        let _b = Database::insert_many::<Bitfield>(&database, &bitfields).await;
        println!("Inserted bitfields");
    }

    if insert_lines {
        println!("Getting lines...");
        let lines: Vec<Line> = hrdf.get_lines().unwrap();
        println!("Got lines: {}", lines.len());

        println!("Inserting lines...");
        let _l = Database::insert_many::<Line>(&database, &lines).await;
        println!("Inserted lines");
    }

    if insert_trips || insert_trip_stops || insert_stops {
        println!("Getting fahrplans...");
        fahrplans = hrdf.get_fahrplans().unwrap();
        println!("Got fahrplans: {}", fahrplans.len());
    }

    if insert_stops {
        println!("Getting stops...");
        let stops_id: Vec<u32> = hrdf.extract_stop_ids(&fahrplans);
        let stops: Vec<Stop> = hrdf.retrieve_stops(stops_id).unwrap();
        println!("Got stops: {}", stops.len());

        println!("Inserting stops...");
        let _s = Database::insert_many::<Stop>(&database, &stops).await;
        println!("Inserted stops");
    }

    if insert_trips {
        println!("Getting trips...");
        let trips: Vec<Trip> = hrdf.to_trips(&fahrplans);
        println!("Got trips: {}", trips.len());

        println!("Inserting trips...");
        let _t = Database::insert_many::<Trip>(&database, &trips).await;
        println!("Inserted trips");
    }

    if insert_trip_stops {
        println!("Getting trip stops...");
        let trip_stops: Vec<TripStop> = hrdf.to_trip_stops(&fahrplans);
        println!("Got trip stops: {}", trip_stops.len());

        println!("Inserting trip stops...");
        let _ts = Database::insert_many::<TripStop>(&database, &trip_stops).await;
        println!("Inserted trip stops");
    }

    // init http server
    HttpServer::new(move || {
        let db_data: Data<Database> = Data::new(database.clone());
        let logger: Logger = Logger::default();
        App::new().app_data(db_data).wrap(logger) //.service(get_line)
    })
    .bind(("127.0.0.1", 10000))?
    .run()
    .await
}
