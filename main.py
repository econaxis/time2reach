# import numpy as np
# import json
# import msgpack
# from PIL import Image, ImageFilter
# import matplotlib
# from matplotlib import pyplot as plt
# import array
#
# data = msgpack.unpackb(open("observations.rmp", "rb").read())
# arr = array.array('i', data["map"]).tolist()
# arr = np.array(arr, dtype = float)
#
# arr[arr==-1] = np.nan
# min_ = np.nanmin(arr)
# max_ = np.nanmax(arr)
#
# arr = (arr - min_)/ (max_ - min_)
# arr = arr
#
# arr = np.reshape(arr, (data["x"], data["y"]))
#
# cmap = matplotlib.colormaps['Spectral']
# arr = cmap(arr)
#
# plt.imshow(arr, origin = "lower")
# plt.colorbar()
# plt.show()
# # img = Image.fromarray(np.uint8(arr * 255))
# # img = img.transpose(method=Image.Transpose.FLIP_TOP_BOTTOM)
# # img = img.filter(ImageFilter.GaussianBlur(radius=5))
# # img.show()
# import fiona
import osmnx
def generate_toronto_geopackage():
    toronto = osmnx.graph_from_point((43.7543,-79.5231), dist = 20000 * 4, network_type="drive_service")
    # polygon = polygon["geometry"].unary_union
    # toronto = osmnx.graph_from_polygon(polygon, network_type="walk")
    osmnx.save_graph_geopackage(toronto, "toronto2.gpkg")
    # map.save("map.html")

generate_toronto_geopackage()