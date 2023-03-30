import mapbox_vector_tile

file = open("sample.pbf", "rb").read()

dec = mapbox_vector_tile.decode(file)
print(dec)