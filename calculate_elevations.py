import functools
import os.path

import math
import requests
from osgeo import osr, gdal, ogr

# API URL for downloading GEOTIFF files

IMAGE_SIZE: int = 2048
PIXEL_SIZE: int = 10  # meters
API_URL: str = "https://elevation.nationalmap.gov/arcgis/rest/services/3DEPElevation/ImageServer/exportImage"
API_PARAMS: dict = {
    "bbox": "-122.543,37.6694,-122.3037,37.8288",
    "size": f"{IMAGE_SIZE},{IMAGE_SIZE}",
    "format": "tiff",
    "pixelType": "F32",
    "noData": "",
    "noDataInterpretation": "esriNoDataMatchAny",
    "interpolation": "RSP_BilinearInterpolation",
    "adjustAspectRatio": "true",
    "lercVersion": "1",
    "f": "image"
}


@functools.lru_cache
def download_geo_tiff(bbox: tuple) -> str:
    API_PARAMS["bbox"] = f"{bbox[0]},{bbox[1]},{bbox[2]},{bbox[3]}"

    # Generate a unique filename based on the bbox
    file_name: str = f"target/geotiff/geotiff_{bbox[0]}_{bbox[1]}_{bbox[2]}_{bbox[3]}_{IMAGE_SIZE}_{PIXEL_SIZE}.tif"

    if os.path.exists(file_name):
        print("Using cached!", bbox)
        return file_name

    # Make the request to download the GeoTIFF file
    response = requests.get(API_URL, params=API_PARAMS)

    # Check if request was successful
    assert response.ok
    # Save the GeoTIFF file to cache
    with open(file_name, 'wb') as f:
        f.write(response.content)
    print(f"GeoTIFF file downloaded and cached for bbox {bbox}.")
    return file_name


def get_ele(lat: float, lon: float, default: float | None) -> float:
    # Convert latitude and longitude to EPSG:3857
    epsg3857_coords = convert_lat_lon_to_epsg3857(lat, lon)
    bbox = calculate_bounding_box(epsg3857_coords[1], epsg3857_coords[0])
    file_name = download_geo_tiff(bbox)

    dataset = open_dataset(file_name)

    # Add logic to extract elevation from downloaded GeoTIFF
    elevation: float = extract_elevation_from_geotiff(dataset, epsg3857_coords[0], epsg3857_coords[1])
    if elevation < 0.1 and default is not None:
        return max(default, elevation)
    return elevation

def convert_lat_lon_to_epsg3857(lat: float, lon: float) -> tuple:
    source = osr.SpatialReference()
    source.ImportFromEPSG(4326)  # EPSG:4326 (WGS 84) for lat/lon

    target = osr.SpatialReference()
    target.ImportFromEPSG(3857)  # EPSG:3857 (Web Mercator)

    transformation = osr.CoordinateTransformation(source, target)

    point = ogr.Geometry(ogr.wkbPoint)
    point.AddPoint(lat, lon)  # Note the order: lon, lat

    point.Transform(transformation)
    return point.GetX(), point.GetY()


def round_to_tile_coordinates(x, y) -> tuple:
    round_factor = IMAGE_SIZE * PIXEL_SIZE
    rounded_lon: float = math.floor(x / round_factor) * round_factor
    rounded_lat: float = math.floor(y / round_factor) * round_factor
    return rounded_lon, rounded_lat


def calculate_bounding_box(lat: float, lon: float) -> tuple:
    lon, lat = round_to_tile_coordinates(lon, lat)

    full_width: float = IMAGE_SIZE * PIXEL_SIZE
    min_x: float = lon
    min_y: float = lat
    max_x: float = lon + full_width
    max_y: float = lat + full_width
    return min_x, min_y, max_x, max_y


@functools.lru_cache
def open_dataset(file_name: str) -> gdal.Dataset:
    # Open the GeoTIFF file
    dataset: gdal.Dataset = gdal.Open(file_name, gdal.GA_ReadOnly)
    if dataset is None:
        raise Exception(f"Failed to open GeoTIFF file: {file_name}")
    return dataset


def extract_elevation_from_geotiff(dataset: gdal.Dataset, x_coord: float, y_coord: float) -> float:
    # Open the GeoTIFF dataset
    # Get the geotransform (affine transformation coefficients)
    geotransform: tuple = dataset.GetGeoTransform()

    # Calculate pixel coordinates
    pixel_x: int = int((x_coord - geotransform[0]) / geotransform[1])
    pixel_y: int = int((y_coord - geotransform[3]) / geotransform[5])

    # Read elevation from the dataset at the specified pixel coordinates
    band: gdal.Band = dataset.GetRasterBand(1)
    elevation = band.ReadAsArray(pixel_x, pixel_y, 1, 1)[0, 0]

    return elevation.item()





import sqlite3
import json

def create_db_and_tables(dbname: str):
    conn = sqlite3.connect(dbname)
    cursor = conn.cursor()

    # Create nodes table
    cursor.execute('''CREATE TABLE IF NOT EXISTS nodes
                      (node_id INTEGER PRIMARY KEY, lat REAL, lon REAL, ele REAL)''')

    # Adjusted edges table to include dist and kvs
    cursor.execute('''CREATE TABLE IF NOT EXISTS edges
                      (id INTEGER PRIMARY KEY, nodeA INTEGER, nodeB INTEGER, dist REAL, kvs TEXT,
                      FOREIGN KEY(nodeA) REFERENCES nodes(node_id),
                      FOREIGN KEY(nodeB) REFERENCES nodes(node_id))''')

    # Adjusted edge_points table to link with edges
    cursor.execute('''CREATE TABLE IF NOT EXISTS edge_points
                      (point_id INTEGER PRIMARY KEY AUTOINCREMENT, edge_id INTEGER, lat REAL, lon REAL, ele REAL,
                       FOREIGN KEY(edge_id) REFERENCES edges(id))''')

    cursor.execute('''CREATE INDEX IF NOT EXISTS idx_edge_points_on_edge_id_and_point_id ON edge_points (edge_id, point_id);''')
    conn.commit()
    conn.close()

def export_edges_to_geojson(db_path, output_path):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Fetch edges with node coordinates
    sql_query_edges = """
    SELECT e.id, nA.lat AS latA, nA.lon AS lonA, nB.lat AS latB, nB.lon AS lonB
    FROM edges AS e
    JOIN nodes AS nA ON e.nodeA = nA.node_id
    JOIN nodes AS nB ON e.nodeB = nB.node_id
    """
    cursor.execute(sql_query_edges)
    edges = cursor.fetchall()

    # Construct GeoJSON for edges
    geojson_edges = {
        "type": "FeatureCollection",
        "features": []
    }

    for edge in edges:
        feature = {
            "type": "Feature",
            "properties": {
                "id": edge[0]
            },
            "geometry": {
                "type": "LineString",
                "coordinates": [
                    [edge[2], edge[1]],  # lonA, latA
                    [edge[4], edge[3]]   # lonB, latB
                ]
            }
        }
        geojson_edges["features"].append(feature)

    # Save edges GeoJSON to file
    with open(output_path, 'w') as f:
        json.dump(geojson_edges, f)

    print(f"Exported edges to {output_path}")


def equiv(param, param1):
    return abs(param - param1) < 0.0001


def add_elevation_to_db(filename: str, dbname: str):
    conn = sqlite3.connect(dbname)
    cursor = conn.cursor()

    # Open the JSON file and load its content
    with open(filename, "r") as f:
        data = json.load(f)

    # Prepare for batch insertion of nodes and edges
    nodes_to_insert = []
    edges_to_insert = []
    edge_points_to_insert = []

    for node in data["nodes"]:
        lat = node["lat"]
        lon = node["lon"]
        elevation = get_ele(lat, lon, node.get("ele"))
        nodes_to_insert.append((node["id"], lat, lon, elevation))

    for edge in data["edges"]:
        edge_id = edge['id']
        kvs_json = json.dumps(edge["kvs"])
        edges_to_insert.append((edge_id, edge['nodeA'], edge['nodeB'], edge['dist'], kvs_json))

        # assert edge["points"][0]["lat"] == data["nodes"][edge['nodeA']]["lat"], f'{edge["points"]} / {data["nodes"][edge["nodeA"]]}'
        # assert edge["points"][0]["lon"] == data["nodes"][edge['nodeA']]["lon"], f'{edge["points"]} / {data["nodes"][edge["nodeA"]]}'

        if not equiv(edge["points"][-1]["lat"], data["nodes"][edge['nodeB']]["lat"]) or not equiv(edge["points"][-1]["lon"], data["nodes"][edge['nodeB']]["lon"]):
            edge["points"].append(data["nodes"][edge['nodeB']])

        for point in edge["points"]:
            lat = point['lat']
            lon = point['lon']
            elevation = get_ele(lat, lon, point.get("ele"))
            edge_points_to_insert.append((edge_id, lat, lon, elevation))

    # Batch insert the prepared data
    cursor.executemany('''INSERT INTO nodes (node_id, lat, lon, ele) VALUES (?, ?, ?, ?)''', nodes_to_insert)
    cursor.executemany('''INSERT INTO edges (id, nodeA, nodeB, dist, kvs) VALUES (?, ?, ?, ?, ?)''', edges_to_insert)
    cursor.executemany('''INSERT INTO edge_points (edge_id, lat, lon, ele) VALUES (?, ?, ?, ?)''', edge_points_to_insert)

    conn.commit()
    conn.close()

if __name__ == "__main__":
    import os
    os.remove("elevation-big.db")
    create_db_and_tables("elevation-big.db")
    add_elevation_to_db("/Users/henry/graphhopper/norcal-big.json", "elevation-big.db")
