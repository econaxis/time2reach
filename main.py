import osmnx
def generate_toronto_geopackage(loc = (43.76592048876812, -79.63720080558336), filename = "toronto2.gpkg", dist = 55000):
    toronto = osmnx.graph_from_point(loc, dist = dist, network_type="drive_service")
    osmnx.save_graph_geopackage(toronto, filename)

cities = ['New York City', 'Vancouver', 'Montreal']

for city in cities:
    print("Working for", city)
    location = osmnx.geocode(city)
    generate_toronto_geopackage(location, f"{city}.gpkg")


