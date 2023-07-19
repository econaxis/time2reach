import pdb
from concurrent.futures import as_completed

from requests_futures.sessions import FuturesSession


def run_profiling():
    import random
    MIN_COORD = (43.70450356508819, -79.50834861469022)
    MAX_COORD = (43.66585904620152, -79.31521611344768)


    # Generate a random number between min and max
    def random_latlong():
        return (random.uniform(MIN_COORD[0], MAX_COORD[0]),
                random.uniform(MIN_COORD[1], MAX_COORD[1]))


    coords = [random_latlong() for _ in range(3000)]
    SERVER_URL = "https://map.henryn.xyz/api/hello"
    with FuturesSession() as session:
        futures = []
        completed = 0
        for coord in coords:

            data = {"latitude":coord[0],"longitude":coord[1],"agencies":["TTC","UP","GO","YRT","BRAMPTON","MIWAY","GRT","NYC-SUBWAY","NYC-BUS","NJ-BUS","NJ-RAIL","VANCOUVER-TRANSLINK","MONTREAL"],"modes":["bus","subway","tram","rail"],"startTime":47035,"maxSearchTime":12500}

            r = session.post(f'{SERVER_URL}', json=data, stream=True)
            futures.append(r)

            if len(futures) > 160:
                for future in as_completed(futures):
                    resp = future.result()
                    resp.raise_for_status()
                    json = resp.json()
                    print("Completed! ", resp, len(json['edge_times']) )
                    completed += 1
                futures = []

        for future in as_completed(futures):
            resp = future.result()
            resp.raise_for_status()
            completed += 1
        session.executor.shutdown(wait = True)


run_profiling()