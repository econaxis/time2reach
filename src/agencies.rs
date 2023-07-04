use crate::gtfs_setup::initialize_gtfs_as_bson;
use crate::gtfs_wrapper::Gtfs1;

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum City {
    #[serde(rename = "New York City")]
    NewYorkCity,
    Vancouver,
    Toronto,
    Montreal,
}

impl FromStr for City {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "newyorkcity" => Ok(City::NewYorkCity),
            "toronto" => Ok(City::Toronto),
            "montreal" => Ok(City::Montreal),
            "vancouver" => Ok(City::Vancouver),
            _ => {
                log::error!("{s} is not a city");
                Err(format!("{s} not a city"))
            }
        }
    }
}

impl City {
    pub fn get_gpkg_path(&self) -> &'static str {
        match self {
            City::NewYorkCity => "NewYorkCity",
            City::Toronto => "Toronto",
            City::Montreal => "Montreal",
            City::Vancouver => "Vancouver",
        }
    }
}

#[derive(Serialize)]
pub struct Agency {
    public_name: &'static str,
    path: &'static str,
    short_code: &'static str,
    city: City,
}

pub fn load_all_gtfs() -> HashMap<City, Gtfs1> {
    let mut result: HashMap<City, Gtfs1> = HashMap::new();

    let agencies_ = agencies();

    print!("Using agencies: ");
    for ag in &agencies_ {
        print!(" {}", ag.public_name);
    }
    println!();

    for agency in agencies_ {
        let this_gtfs =
            initialize_gtfs_as_bson(&format!("city-gtfs/{}", agency.path), agency.short_code);
        if let Some(gtfs) = result.get_mut(&agency.city) {
            gtfs.merge(this_gtfs)
        } else {
            result.insert(agency.city, this_gtfs);
        }
    }

    result
}

pub fn agencies() -> Vec<&'static Agency> {
    const AGENCY_TORONTO: [Agency; 11] = [
        Agency {
            public_name: "TTC",
            path: "ttc",
            short_code: "TTC",
            city: City::Toronto,
        },
        Agency {
            public_name: "UP Express",
            city: City::Toronto,
            path: "up_express",
            short_code: "UP",
        },
        Agency {
            public_name: "GO Transit",
            city: City::Toronto,
            path: "GO_GTFS",
            short_code: "GO",
        },
        Agency {
            public_name: "York Region Transit",
            city: City::Toronto,
            path: "yrt",
            short_code: "YRT",
        },
        Agency {
            public_name: "Brampton Transit",
            city: City::Toronto,
            path: "brampton",
            short_code: "BRAMPTON",
        },
        Agency {
            public_name: "Miway (Mississauga)",
            city: City::Toronto,
            path: "miway",
            short_code: "MIWAY",
        },
        Agency {
            public_name: "GRT (Kitchener/Waterloo)",
            city: City::Toronto,
            path: "waterloo_grt",
            short_code: "GRT",
        },
        // New York City
        Agency {
            public_name: "MTA Subway",
            city: City::NewYorkCity,
            path: "nyc-subway",
            short_code: "NYC-SUBWAY",
        },
        Agency {
            public_name: "MTA Bus",
            city: City::NewYorkCity,
            path: "nyc-bus",
            short_code: "NYC-BUS",
        },
        // Vancouver
        Agency {
            public_name: "Vancouver Translink",
            city: City::Vancouver,
            path: "vancouver-translink",
            short_code: "VANCOUVER-TRANSLINK",
        },
        // Montreal
        Agency {
            public_name: "Montreal STM",
            city: City::Montreal,
            path: "montreal",
            short_code: "MONTREAL",
        },
    ];

    if cfg!(feature = "all-cities") {
        AGENCY_TORONTO.iter().collect()
    } else {
        AGENCY_TORONTO
            .iter()
            .filter(|x| x.city == City::Toronto)
            .take(3)
            .collect()
    }
}
