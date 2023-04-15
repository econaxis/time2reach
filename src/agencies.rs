use std::collections::HashMap;
use std::str::FromStr;
use serde::Serialize;
use crate::gtfs_setup::initialize_gtfs_as_bson;
use crate::gtfs_wrapper::Gtfs1;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Serialize)]
pub enum City {
    NewYorkCity,
    Vancouver,
    Toronto,
    Montreal,
}

impl FromStr for City {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NewYorkCity" => Ok(City::NewYorkCity),
            "Toronto" => Ok(City::Toronto),
            "Montreal" => Ok(City::Montreal),
            "Vancouver" => Ok(City::Vancouver),
            _ => Err(format!("{s} not a city"))
        }
    }
}

impl City {
    pub fn get_gpkg_path(&self) -> &'static str {
        match self {
            City::NewYorkCity => "NewYorkCity",
            City::Toronto => "Toronto",
            City::Montreal => "Montreal",
            City::Vancouver => "Vancouver"
        }
    }
}

#[derive(Serialize)]
struct Agency {
    public_name: &'static str,
    path: &'static str,
    short_code: &'static str,
    city: City,
}


pub fn load_all_gtfs() -> HashMap<City, Gtfs1> {
    let mut result: HashMap<City, Gtfs1> = HashMap::new();

    for agency in agencies() {
        let this_gtfs = initialize_gtfs_as_bson(
            &format!("/Users/henry.nguyen@snapcommerce.com/Downloads/{}", agency.path),
            agency.short_code,
        );
        if let Some(gtfs) = result.get_mut(&agency.city) {
            gtfs.merge(this_gtfs)
        } else {
            result.insert(agency.city, this_gtfs);
        }
    }

    result
}

const fn agencies<'a>() -> &'a [Agency] {
    return &[
        Agency {
            public_name: "TTC",
            path: "ttc",
            short_code: "ttc",
            city: City::Toronto,
        },
        Agency {
            public_name: "UP Express",
            city: City::Toronto,
            path: "up_express",
            short_code: "UP",
        },
        Agency {
            public_name: "GRT (Kitchener/Waterloo)",
            city: City::Toronto,
            path: "waterloo_grt",
            short_code: "GRT",
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

        // New York City
        Agency {
            public_name: "NYC Subway",
            city: City::NewYorkCity,
            path: "nyc-subway",
            short_code: "nyc-subway",
        },
        Agency {
            public_name: "NYC Bus",
            city: City::NewYorkCity,
            path: "nyc-bus",
            short_code: "nyc-bus",
        },
    ];
}