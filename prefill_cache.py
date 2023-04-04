import dataclasses
import math
import queue

import requests
import sys
from requests_futures.sessions import FuturesSession

url = sys.argv[1]

@dataclasses.dataclass
class Explore:
    zoom: int
    x: int
    y: int

to_explore = queue.Queue()
to_explore.put(Explore(8, 71, 93))
to_explore.put(Explore(8, 72, 93))
to_explore.put(Explore(8, 72, 92))
to_explore.put(Explore(8, 70, 93))

to_explore_calculated = []
MAX_ZOOM = 12
while not to_explore.empty():
    explore = to_explore.get_nowait()
    to_explore_calculated.append(explore)
    # Calculate tiles to max zoom
    for cur_zoom in range(explore.zoom + 1, MAX_ZOOM + 1):
        zoom_diff = cur_zoom - explore.zoom
        multiplier = int(math.pow(2, zoom_diff))

        to_explore.put(Explore(cur_zoom, multiplier * explore.x, multiplier * explore.y))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x + 1, multiplier * explore.y))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x, multiplier * explore.y + 1))
        to_explore.put(Explore(cur_zoom, multiplier * explore.x + 1, multiplier * explore.y + 1))


session = FuturesSession()
for coord in to_explore_calculated:
    r = session.get(f'{url}/{coord.zoom}/{coord.x}/{coord.y}.pbf')
    print(r)
