mod api;
mod model;
mod repository;

use std::{env, path::Path};

//use api::line::get_line;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use model::{
    line::{Line, TransportMode},
    bitfield::Bitfield,
    stop::Stop,
    trip::Trip,
};
use repository::{
    database::Database,
    gtfs::{RawHaltestellen, GTFS},
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

    let hrdf: HRDF = HRDF {
        directory: Path::new(&env::var("HRDF_PATH").unwrap().parse::<String>().unwrap()).to_path_buf(),
        agency_id: env::var("AGENCY_ID").unwrap().parse::<u32>().unwrap(),
    };
    let fahrplans: Vec<Fahrplan> = hrdf.get_fahrplans().unwrap();
    println!("Got fahrplans !");
    //println!("{:#?}", fahrplans);
    /*let lines: Vec<Line> = hrdf.get_lines().unwrap();
    println!("{:#?}", lines);*/
    /*let bitfields: Vec<Bitfield> = hrdf.get_bitfields().unwrap();
    println!("{:#?}", bitfields);*/
    let trips: Vec<Trip> = hrdf.to_trips(fahrplans);
    println!("{:#?}", trips);

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
