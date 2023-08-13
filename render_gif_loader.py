import json
import os.path

import flask
import requests

# NY Penn station latitude longitude
# 40.750580, -73.993584
BODY = {"latitude":40.750580,"longitude":-73.993584, "agencies":["MTA New York City Transit","MTA Bus Company","NJ TRANSIT BUS","NJ TRANSIT RAIL","Port Authority Trans-Hudson Corporation","Union City Transit","Capitol Corridor Joint Powers Authority","San Francisco International Airport","Altamont Corridor Express","Emery Go-Round","Petaluma","Mission Bay TMA","Santa Rosa CityBus","MVgo Mountain View","Angel Island Tiburon Ferry","SolTrans","County Connection","Dumbarton Express Consortium","San Francisco Municipal Transportation Agency","AC TRANSIT","Marin Transit","VINE Transit","Livermore Amador Valley Transit Authority","Sonoma County Transit","Sonoma Marin Area Rail Transit","Treasure Island Ferry","Tri Delta Transit","Golden Gate Ferry","Golden Gate Transit","City of South San Francisco","San Francisco Bay Ferry","Commute.org Shuttles","Caltrain","SamTrans","VTA","Bay Area Rapid Transit","TTC","UP Express","GO Transit","York Region Transit","Brampton Transit","MiWay","GRT","Société de transport de Montréal","TransLink","Chicago Transit Authority","RER","Noctilien","TER","Transilien","Terres d'Envol","Poissy - Les Mureaux","Phébus","Paris-Saclay Mobilités","Seine-Saint-Denis","Bus Haut Val d'Oise","Sit'bus","Grand Melun","Valoise","Chavilbus","Seine Essonne Bus","Aérial","Tam Limay","Saint Germain Boucles de Seine","Argenteuil - Boucles de Seine","Génovébus","SITUS","Vélizy Vallées","Arlequin","Saint-Quentin-en-Yvelines","Plaine de Versailles","Meaux et Ourcq","Goëlys","ValBus","Paris Saclay","Traverciel","Apolo 7","Val de Seine","Orgebus","Brie et 2 Morin","Vallée Grand Sud Paris","Sénart","Marne et Seine","Vexin","Vallée de Montmorency","Bièvre","Val d'Yerres Val de Seine","Comète","Bassin de Claye","Essonne Sud Est","Roissy Ouest","Siyonne","Conflans Achères","Seine et Marne Express","Filéo","Essonne Sud Ouest","Réseau du Canton de Perthes","Busval d'Oise","STILL","Titus","Pays Briard","Houdanais","Rambouillet Urbain","Parisis","Mantois","Marne-la-Vallée","STILE Express","Rambouillet Interurbain","Scolaire Est Yvelines","Seine Grand Orly","RATP","Transdev CEAT","Keolis Meyer","CIF","Trans Val d'Oise","Aéroport Paris-Beauvais / SAGEB","Cars Lacroix","Cars Moreau","Transdev Ile-de-France Lys","Cars Rose","SAVAC","Transdev Ile-de-France Vulaines","Stivo","Magical Shuttle","TICE","ADP","ProCars","Transdev Autocars Tourneux","Autobus du Fort","Cars Soeur","Transdev Valmy","Darche Gros","Keolis Mobilité Roissy","Transdev Ile-de-France Conflans","Francilité Grand Provinois","Albatrans","Keolis Ouest Val-de-Marne","Cars Hourtoule","STAVO","Transdev CSO","Les Cars Bleus","SETRA","Cars Losay","Mobicité","Autocars Dominique","Keolis Val d'Oise"],"modes":["bus","subway","tram","rail","ferry"],"startTime":18060,"maxSearchTime":2700}
START_TIME = 3 * 3600

# Flask simple file loader
app = flask.Flask(__name__)

@app.route("/<starttime>")
def index(starttime: int):
    # Set cors headers
    response = flask.Response(
        open(f"/tmp/responsenyc{starttime}.txt", "r").read(),
        mimetype="application/json",
    )
    response.headers["Access-Control-Allow-Origin"] = "*"
    return response

app.run(host="127.0.0.1", port = 8000)

if __name__ == "__main__":
    while START_TIME < 5.1 * 3600:
        print("starting ", START_TIME)
        if os.path.exists(f"/tmp/responsenyc{START_TIME}.txt"):
            START_TIME += 60
            continue


        BODY["startTime"] = START_TIME

        # Convert the above to python requests
        response = requests.post(
            "https://api-map-v2.henryn.ca/hello/",
            json=BODY,
        )

        open(f"/tmp/responsenyc{START_TIME}.txt", "w").write(response.text)

        START_TIME += 60

