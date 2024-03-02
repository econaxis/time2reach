import os.path

import math
import json

from osgeo import osr, gdal, ogr
import requests
import functools

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


def get_ele(lat: float, lon: float) -> float:
    # Convert latitude and longitude to EPSG:3857
    epsg3857_coords = convert_lat_lon_to_epsg3857(lat, lon)
    bbox = calculate_bounding_box(epsg3857_coords[1], epsg3857_coords[0])
    file_name = download_geo_tiff(bbox)

    dataset = open_dataset(file_name)

    # Add logic to extract elevation from downloaded GeoTIFF
    elevation: float = extract_elevation_from_geotiff(dataset, epsg3857_coords[0], epsg3857_coords[1])
    return round(elevation, 1)


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


def add_elevation_to_json(filename: str):
    with open(filename, "r") as f:
        data = json.load(f)

    for node in data["nodes"]:
        lat = node["lat"]
        lon = node["lon"]
        elevation = get_ele(lat, lon)
        node["ele"] = elevation

    for edge in data["edges"]:
        for points in edge["points"]:
            lat = points['lat']
            lon = points['lon']
            elevation = get_ele(lat, lon)
            points['ele'] = elevation

    with open(filename, "w") as f:
        json.dump(data, f)

def main():
    latitude: float = 37.76527
    longitude: float = -122.44077
    elevation: float = get_ele(latitude, longitude)
    elevation1: float = get_ele(latitude + 0.001, longitude + 0.001)
    if elevation is not None:
        print(f"Elevation at ({latitude}, {longitude}): {elevation} meters")
        print(f"Elevation at ({latitude}, {longitude}): {elevation1} meters")
    else:
        print("Failed to retrieve elevation.")


if __name__ == "__main__":
    add_elevation_to_json("/Users/henry/graphhopper/norcal-small.json")
