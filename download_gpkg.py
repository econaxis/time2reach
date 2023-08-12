import json
import os
import subprocess

import osmnx
import psycopg2 as psycopg2
import geopandas as gpd
import shapely.geometry
from shapely.geometry import Point

SAN_FRAN = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": [
                    [
                        [
                            -122.4402332267443,
                            37.8765048081046
                        ],
                        [
                            -122.60042367130171,
                            37.74439722297838
                        ],
                        [
                            -122.45608525580958,
                            37.504772808207335
                        ],
                        [
                            -122.19904107804977,
                            37.32587578924125
                        ],
                        [
                            -121.96687214329903,
                            37.206592881014416
                        ],
                        [
                            -121.70256827084697,
                            37.20606820178594
                        ],
                        [
                            -121.84249592825383,
                            37.51528280385597
                        ],
                        [
                            -121.96935966759973,
                            37.64653239082193
                        ],
                        [
                            -122.09041918357691,
                            37.7657676092272
                        ],
                        [
                            -122.34769528261083,
                            38.03084389715397
                        ],
                        [
                            -122.4402332267443,
                            37.8765048081046
                        ]
                    ]
                ],
                "type": "Polygon"
            }
        }
    ]
}

PARIS = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": [
                    [
                        [
                            2.386644608328851,
                            49.039690902033186
                        ],
                        [
                            2.2620114292866162,
                            49.060761666476225
                        ],
                        [
                            2.0134019029884627,
                            49.065225553888
                        ],
                        [
                            1.9453761535031617,
                            48.92687950313834
                        ],
                        [
                            2.021035401192904,
                            48.748331333247506
                        ],
                        [
                            2.174201713512417,
                            48.77857261382735
                        ],
                        [
                            2.2541909507718003,
                            48.71009469528801
                        ],
                        [
                            2.2233179118289286,
                            48.63317734582276
                        ],
                        [
                            2.2976938692811757,
                            48.57935816820455
                        ],
                        [
                            2.425304755208032,
                            48.568141111506606
                        ],
                        [
                            2.5572526492323107,
                            48.553081756755944
                        ],
                        [
                            2.691173969469361,
                            48.67127416235684
                        ],
                        [
                            2.642949708080266,
                            48.808041430512844
                        ],
                        [
                            2.6440311641975143,
                            48.892725127108264
                        ],
                        [
                            2.618076217386374,
                            48.96164707681481
                        ],
                        [
                            2.5272339035514904,
                            49.02196440719797
                        ],
                        [
                            2.386644608328851,
                            49.039690902033186
                        ]
                    ]
                ],
                "type": "Polygon"
            }
        }
    ]
}
MEXICO_CITY = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": [
                    [
                        [
                            -99.25558506970417,
                            19.595663425040556
                        ],
                        [
                            -99.31657147783135,
                            19.44680302075662
                        ],
                        [
                            -99.23056500483145,
                            19.321418511249234
                        ],
                        [
                            -99.10807093722536,
                            19.286982369589623
                        ],
                        [
                            -99.01945820746784,
                            19.42566652180362
                        ],
                        [
                            -99.01163943719546,
                            19.565705859492596
                        ],
                        [
                            -99.07366834802558,
                            19.63101600013458
                        ],
                        [
                            -99.20502368860747,
                            19.633470751348227
                        ],
                        [
                            -99.25558506970417,
                            19.595663425040556
                        ]
                    ]
                ],
                "type": "Polygon"
            }
        }
    ]
}


LONDON = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": [
                    [
                        [
                            -0.49363863553605825,
                            51.477859393329226
                        ],
                        [
                            -0.2551692312244995,
                            51.45531018231321
                        ],
                        [
                            -0.10541612315978455,
                            51.395567295407346
                        ],
                        [
                            0.06278997452344015,
                            51.432749826959736
                        ],
                        [
                            0.12524624708169085,
                            51.51850822356468
                        ],
                        [
                            -0.03302362542206083,
                            51.61248001739568
                        ],
                        [
                            -0.39285692300015285,
                            51.590879292163436
                        ],
                        [
                            -0.49363863553605825,
                            51.477859393329226
                        ]
                    ]
                ],
                "type": "Polygon"
            }
        }
    ]
}

CHICAGO = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": [
                    [
                        [
                            -87.67258140774001,
                            42.086068447829064
                        ],
                        [
                            -87.75125111112669,
                            42.10198931558017
                        ],
                        [
                            -87.82634491890458,
                            42.086068447829064
                        ],
                        [
                            -87.86746771840207,
                            42.027657743045324
                        ],
                        [
                            -87.87223557921354,
                            41.97273814749306
                        ],
                        [
                            -87.88355924864038,
                            41.932404096903525
                        ],
                        [
                            -87.8930949702628,
                            41.864088334649836
                        ],
                        [
                            -87.88057933563327,
                            41.822797322673836
                        ],
                        [
                            -87.87342754441629,
                            41.777479769711874
                        ],
                        [
                            -87.80131364964548,
                            41.70543897070036
                        ],
                        [
                            -87.688334551329,
                            41.63112372443214
                        ],
                        [
                            -87.58880545689306,
                            41.627559948784295
                        ],
                        [
                            -87.45470937157499,
                            41.619095192064464
                        ],
                        [
                            -87.38676735501352,
                            41.67877028361326
                        ],
                        [
                            -87.43623391093121,
                            41.71348107581585
                        ],
                        [
                            -87.49940806668067,
                            41.70235786180757
                        ],
                        [
                            -87.57458860827903,
                            41.83365128339878
                        ],
                        [
                            -87.60379175574855,
                            41.93126933443378
                        ],
                        [
                            -87.67258140774001,
                            42.086068447829064
                        ]
                    ]
                ],
                "type": "Polygon"
            }
        }
    ]
}
cities = [
    # (CHICAGO, 'Chicago'),
    # (PARIS, 'Paris'),
    # (LONDON, 'London'),
    # (MEXICO_CITY, 'MexicoCity'),
    (SAN_FRAN, 'SanFrancisco'),
    ]




def create_circle_polygon(long, lat, radius):
    point = Point(long, lat)
    # Create a buffer around the point using the radius to generate a circle
    circle = point.buffer(radius)

    # Convert the circle to a GeoPandas polygon
    polygon = gpd.GeoSeries(circle)

    return polygon

import shapely
def create_poly_from_geojson(geojson: dict):
    # Create a GeoDataFrame from the GeoJSON data
    shape = shapely.geometry.shape(geojson['features'][0]['geometry'])
    return shape
def generate_toronto_geopackage(loc = (43.76592048876812, -79.63720080558336), filename = "toronto2.gpkg", dist = 33000):
    if type(loc) == dict:
        polygon = create_poly_from_geojson(loc)
    else:
        polygon = create_circle_polygon(loc[1], loc[0], dist)
    toronto = osmnx.graph_from_polygon(polygon, network_type="drive_service")
    print("Saving...")
    osmnx.save_graph_geopackage(toronto, filename)


def generate_geopackage_all_cities():
    for (location, filename) in cities:
        print("Working for", filename)
        # location = osmnx.geocode(city)
        generate_toronto_geopackage(location, f"web/public/{filename}.gpkg")


def load_geopackage_to_postgis():
    geopackages = ["NewYorkCity", "Vancouver", "Montreal", "Toronto"]
    geopackages += ["Paris", "London", "MexicoCity", "SanFrancisco", "Chicago"]
    pg_connection_string = "postgresql://localhost:5432/data"
    libpq_connection_string = "dbname=data host=localhost port=5432"
    #
    path_executable = "/opt/homebrew/bin/ogr2ogr"
    command = """-f PostgreSQL "{conn_string}" web/public/{cityname}.gpkg -nln {cityname} -preserve_fid -overwrite edges"""

    for city in geopackages:
        commandrun = command.format(conn_string = pg_connection_string, cityname = city)
        subprocess.run(path_executable + " " + commandrun, shell=True, check=True)

    db = psycopg2.connect(libpq_connection_string)
    cursor = db.cursor()

    print("Done inserting geopackages")
    select_statements = [
        f"SELECT fid, u, v, geom FROM {city}" for city in geopackages
    ]

    select = " UNION ALL ".join(select_statements)

    cursor.execute(f"""
    INSERT INTO all_cities  
    {select}
    """)

    cursor.execute("create index if not exists all_cities_index_geom ON all_cities USING gist(geom)")
    db.commit()

if __name__ =="__main__":
    generate_geopackage_all_cities()
    load_geopackage_to_postgis()
