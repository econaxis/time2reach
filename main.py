import osmnx
def generate_toronto_geopackage():
    toronto = osmnx.graph_from_point((43.76592048876812, -79.63720080558336), dist = 85000, network_type="drive_service")
    # polygon = polygon["geometry"].unary_union
    # toronto = osmnx.graph_from_polygon(polygon, network_type="walk")
    osmnx.save_graph_geopackage(toronto, "toronto2.gpkg")
    # map.save("map.html")

generate_toronto_geopackage()

def generate_pyosm():
    fp = pyrosm.get_data("Ontario")
    osm = pyrosm.OSM(fp)
    driving = osm.get_network(network_type = "driving")

