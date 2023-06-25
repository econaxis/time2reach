import dataclasses
import os.path
import pdb
from concurrent.futures import as_completed

import math
import queue

import sys
from requests_futures.sessions import FuturesSession

url = sys.argv[1]


def lat_lon_to_tile(lat, lon, zoom):
    x = (lon + 180) / 360 * (2 ** zoom)
    lat_rad = math.radians(lat)
    y = (1 - math.log(math.tan(lat_rad) + 1 / math.cos(lat_rad)) / math.pi) / 2 * (2 ** zoom)
    return int(x), int(y)


@dataclasses.dataclass
class Explore:
    zoom: int
    x: int
    y: int

    @classmethod
    def from_latlong(cls, lat: float, lng: float, zoom: int):
        x, y = lat_lon_to_tile(lat, lng, zoom)
        return Explore(zoom, x, y)

    def __hash__(self):
        return hash((self.zoom, self.x, self.y))


to_explore = queue.Queue()

VANCOUVER_TL = (49.328910726698005, -123.25959613136175)
VANCOUVER_BR = (49.15112007739324, -122.86397283811988)

MONTREAL_1 = (45.64534888679888, -73.5359544928739)
MONTREAL_2 = (45.430926507657276, -73.76828002313724)

NYC_1 = (40.64775151884592, -74.1134681369716)
NYC_2 = (40.94121411047778, -73.78936332225874)

# to_explore.put(Explore.from_latlong(*NYC_1, 7))
to_explore.put(Explore.from_latlong(*NYC_2, 7))

to_explore_calculated = set()
MAX_ZOOM = 16
while not to_explore.empty():
    explore = to_explore.get_nowait()
    to_explore_calculated.add(explore)
    # Calculate tiles to max zoom
    for cur_zoom in range(explore.zoom + 1, MAX_ZOOM + 1):
        zoom_diff = cur_zoom - explore.zoom
        multiplier = int(math.pow(2, zoom_diff))

        to_explore.put(Explore(cur_zoom, multiplier * explore.x, multiplier * explore.y))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x + 1, multiplier * explore.y))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x, multiplier * explore.y + 1))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x + 1, multiplier * explore.y + 1))


def pre_check(coord: Explore):
    return os.path.exists(f"/tmp/vancouver-cache/all_cities/{coord.zoom}/{coord.x}/{coord.y}.pbf")
with FuturesSession() as session:
    futures = []
    completed = 0
    for coord in to_explore_calculated:
        assert coord.zoom >= 7

        if pre_check(coord):
            print("Skipping passed pre-check")
            completed += 1
            continue
        r = session.get(f'{url}/{coord.zoom}/{coord.x}/{coord.y}.pbf')
        futures.append(r)

        if len(futures) > 50:
            for future in as_completed(futures):
                resp = future.result()
                resp.raise_for_status()
                print("Completed! ", resp, len(resp.content), f"{completed} out of {len(to_explore_calculated)}")
                completed += 1
            futures = []

    for future in as_completed(futures):
        resp = future.result()
        resp.raise_for_status()
        print("Completed! ", resp, len(resp.content), f"{completed} out of {len(to_explore_calculated)}")
        completed += 1
    session.executor.shutdown(wait = True)
