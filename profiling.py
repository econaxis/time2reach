import time
from concurrent.futures import as_completed

from requests_futures.sessions import FuturesSession
import requests
requests.packages.urllib3.disable_warnings(requests.packages.urllib3.exceptions.InsecureRequestWarning)
def run_profiling():
    import random
    MIN_COORD = (43.70450356508819, -79.50834861469022)
    MAX_COORD = (43.66585904620152, -79.31521611344768)


    # Generate a random number between min and max
    def random_latlong():
        return (random.uniform(MIN_COORD[0], MAX_COORD[0]),
                random.uniform(MIN_COORD[1], MAX_COORD[1]))


    coords = [random_latlong() for _ in range(10000)]
    t = time.time()
    # SERVER_URL = "https://map.henryn.xyz/api/hello"
    SERVER_URL = "https://35.239.72.124/hello"
    with FuturesSession() as session:
        futures = []
        completed = 0
        for coord in coords:

            search_time = random.uniform(2.0 * 3600, 2.20 * 3600)
            data = {"latitude":coord[0],"longitude":coord[1],"agencies":["TTC","UP","GO","YRT","BRAMPTON","MIWAY","GRT","NYC-SUBWAY","NYC-BUS","NJ-BUS","NJ-RAIL","VANCOUVER-TRANSLINK","MONTREAL"],"modes":["bus","subway","tram","rail"],"startTime":47035,"maxSearchTime":search_time}

            r = session.post(f'{SERVER_URL}', json=data, stream=True, verify=False)
            futures.append(r)

            if len(futures) > 2000:
                for future in as_completed(futures):
                    resp = future.result()
                    resp.raise_for_status()
                    json = resp.json()
                    completed += 1
                    print("Completed! ", resp, len(json['edge_times']), (completed / (time.time() - t)))
                futures = []

        for future in as_completed(futures):
            resp = future.result()
            resp.raise_for_status()
            completed += 1
        session.executor.shutdown(wait = True)


run_profiling()