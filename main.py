import numpy as np
import json
import msgpack
from PIL import Image, ImageFilter
import matplotlib
import array

data = msgpack.unpackb(open("observations.rmp", "rb").read())
arr = array.array('i', data["map"]).tolist()
arr = np.array(arr, dtype = float)

arr[arr==-1] = np.nan
min_ = np.nanmin(arr)
max_ = np.nanmax(arr)

arr = (arr - min_)/ (max_ - min_)
arr = 1 - arr

arr = np.reshape(arr, (data["x"], data["y"]))

cmap = matplotlib.colormaps['Spectral']
arr = cmap(arr)

img = Image.fromarray(np.uint8(arr * 255))
img = img.transpose(method=Image.Transpose.FLIP_TOP_BOTTOM)
img = img.filter(ImageFilter.GaussianBlur(radius=5))
img.show()