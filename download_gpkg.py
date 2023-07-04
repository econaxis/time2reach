import os
import subprocess

import osmnx
import psycopg2 as psycopg2


cities = [
    ('Paris, France', 'Paris'),
    ()
    ]
def generate_toronto_geopackage(loc = (43.76592048876812, -79.63720080558336), filename = "toronto2.gpkg", dist = 55000):
    toronto = osmnx.graph_from_point(loc, dist = dist, network_type="drive_service")
    osmnx.save_graph_geopackage(toronto, filename)


def generate_geopackage_all_cities():
    for city in cities:
        print("Working for", city)
        location = osmnx.geocode(city)
        generate_toronto_geopackage(location, f"{city}.gpkg")


def load_geopackage_to_postgis():
    geopackages = ["NewYorkCity", "Vancouver", "Montreal", "Toronto"]
    pg_connection_string = "postgresql://localhost:5432/data"
    libpq_connection_string = "dbname=data host=localhost port=5432"

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
    CREATE TABLE all_cities AS 
    {select}
    """)

    cursor.execute("create index all_cities_index_geom ON all_cities USING gist(geom)")
    db.commit()

if __name__ =="__main__":
    load_geopackage_to_postgis()
