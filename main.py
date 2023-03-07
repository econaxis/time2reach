import osmnx
def generate_toronto_geopackage():
    toronto = osmnx.graph_from_point((43.7543,-79.5231), dist = 20000 * 4, network_type="drive_service")
    # polygon = polygon["geometry"].unary_union
    # toronto = osmnx.graph_from_polygon(polygon, network_type="walk")
    osmnx.save_graph_geopackage(toronto, "toronto2.gpkg")
    # map.save("map.html")

generate_toronto_geopackage()