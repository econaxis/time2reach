use crate::gtfs_setup::initialize_gtfs_as_bson;
use gtfs_structure_2::gtfs_wrapper::Gtfs1;

use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, Debug)]
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
    pub fn get_city_center(&self) -> [f64; 2] {
        let coords = match self {
            City::NewYorkCity => (40.7128, -74.0060), // Center location of New York City (latitude, longitude)
            City::Vancouver => (49.2827, -123.1207), // Center location of Vancouver (latitude, longitude)
            City::Toronto => (43.6532, -79.3832), // Center location of Toronto (latitude, longitude)
            City::Montreal => (45.5017, -73.5673), // Center location of Montreal (latitude, longitude)
        };

        [coords.0, coords.1]
    }
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

pub fn load_all_gtfs() -> FxHashMap<City, Gtfs1> {
    let mut result: FxHashMap<City, Gtfs1> = FxHashMap::default();

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
    const AGENCY_TORONTO: [Agency; 13] = [
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
        Agency {
            public_name: "New Jersey Bus",
            city: City::NewYorkCity,
            path: "nj-bus",
            short_code: "NJ-BUS",
        },
        Agency {
            public_name: "New Jersey Train",
            city: City::NewYorkCity,
            path: "nj-rail",
            short_code: "NJ-RAIL",
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
            .take(1)
            .collect()
    }
}
