import './style.css'
import Geometry from 'geojson'
import {
    Canvas,
    FeatureTiles,
    GeoPackageAPI,
    NumberFeaturesTile,
    ProjectionConstants,
    setSqljsWasmLocateFile,
    TileBoundingBoxUtils
} from "@ngageoint/geopackage";
import L from "leaflet"
import {CrsGeometry} from "@ngageoint/geopackage/dist/lib/types/CrsGeometry";

import createColorMap from 'colormap'

setSqljsWasmLocateFile(filename => `https://unpkg.com/@ngageoint/geopackage@4.2.3/dist/` + filename);
var map = L.map('map').setView([43.657628, -79.450641], 13);

const tileCache = new Map();

window.tileCache = tileCache;

map.on('click', (evt) => {
    const body = {
        latitude: evt.latlng.lat,
        longitude: evt.latlng.lng
    };
    tileCache.clear();
    console.log("Loading...");
    load_gpkg(body).then(() => {
        console.log("Done loading new coords", evt.latlng)
    })
})

const cmap = createColorMap({
    alpha: 0.4,
    colormap: 'jet',
    format: 'hex',
    nshades: 100
});



function gen_key(x, y, z) {
    return `${x}_${y}_${z}`
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function drawTile(ft: FeatureTiles, x: number, y: number, z: number, tileProjection, tileCanvas: any, color_data: ColorsData) {
    const context = tileCanvas.getContext('2d');
    const width = ft.tileWidth;
    const height = ft.tileHeight;
    context.clearRect(0, 0, width, height);
    if (tileCache.get(gen_key(x, y, z))) {
        console.log("Using cache for", x, y, z);
        const imgdata = tileCache.get(gen_key(x, y, z));
        context.putImageData(imgdata, 0, 0);
        return;
    }

    const boundingBox =
        tileProjection === ProjectionConstants.EPSG_3857
            ? TileBoundingBoxUtils.getWebMercatorBoundingBoxFromXYZ(x, y, z)
            : TileBoundingBoxUtils.getWGS84BoundingBoxFromXYZ(x, y, z);
    const expandedBoundingBox = ft.expandBoundingBox(boundingBox, tileProjection);


    let zoom = map.getZoom();

    let modulo_cutoff = 1;

    if (zoom >= 12) {
        modulo_cutoff = 1;
    } else if (zoom >= 11) {
        modulo_cutoff = 2;
    } else if (zoom >= 10) {
        modulo_cutoff = 8;
    } else if (zoom >= 8) {
        modulo_cutoff = 18;
    }

    // get number of features that could intercept this bounding box
    const featureCount = ft.featureDao.countInBoundingBox(expandedBoundingBox, tileProjection);
    ft.linePaint.strokeWidth = 4;
    if (featureCount > 0) {
        if (ft.maxFeaturesPerTile == null || featureCount <= ft.maxFeaturesPerTile) {
            const transform = ft.getTransformFunction(tileProjection);
            const iterator = ft.featureDao.fastQueryBoundingBox(expandedBoundingBox, tileProjection);
            for (const featureRow of iterator) {
                if (Math.random() < 0.004) {
                    await sleep(2);
                }
                if (featureRow.values?.["highway"] == "service" && featureRow.values["u"] % modulo_cutoff != 0) {
                    continue;
                }
                if (featureRow.values?.["highway"] == "residential" && featureRow.values["u"] % modulo_cutoff != 0) {
                    continue;
                }
                if (featureRow.geometry != null) {
                    let geojson = null;
                    if (ft.cacheGeometries) {
                        geojson = ft.geometryCache.getGeometry(featureRow.id);
                    }
                    if (geojson == null) {
                        geojson = featureRow.geometry.geometry.toGeoJSON() as Geometry & CrsGeometry;
                        ft.geometryCache.setGeometry(featureRow.id, geojson);
                    }
                    const style = ft.getFeatureStyle(featureRow);


                    const from_node = <number>featureRow.values?.["from"];
                    const to_node = <number>featureRow.values?.["to"];

                    let time;
                    if (color_data.m[from_node] && color_data.m[to_node]) {
                        time = (color_data.m[from_node] + color_data.m[to_node]) / 2;
                    } else {
                        time = null;
                    }
                    if (time === null || time === undefined) {
                        ft.linePaint.color = "#A7727244"
                    } else {
                        let time_mapped = (time - color_data.min) / (color_data.max - color_data.min);
                        time_mapped = Math.round(time_mapped * 100);

                        time_mapped = Math.min(99, time_mapped);
                        time_mapped = Math.max(0, time_mapped);
                        ft.linePaint.color = cmap[time_mapped] + '99';
                    }

                    try {
                        await ft.drawGeometry(geojson, context, boundingBox, style, transform);
                    } catch (e) {
                        console.error(
                            'Failed to draw feature in tile. Id: ' + featureRow.id + ', Table: ' + ft.featureDao.table_name,
                        );
                    }
                }
            }
        }
    }

    tileCache.set(gen_key(x, y, z), context.getImageData(0, 0, width, height));
}

class ColorsData {
    m: Map<number, number>
    min: number
    max: number

    constructor() {
        this.m = new Map();
        this.min = Infinity;
        this.max = -Infinity;
    }
}
async function get_data(body) : Promise<ColorsData> {
    const data = await fetch("http://localhost:3030/hello", {
        method: 'POST',
        mode: 'cors',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
    });
    const js = await data.json();

    const colors = new ColorsData();
      colors.m = js
    for (const nodeid in js) {
        const time = js[nodeid];
        colors.min = Math.min(colors.min, time);
        colors.max = Math.max(colors.max, time);
    }
    return colors;
}

let previous_layer = null;

async function load_gpkg(body) {
    const gpkg = await GeoPackageAPI.open("/toronto2.gpkg");
    const color_data = await get_data(body);
    window.data = color_data;
    await Canvas.initializeAdapter();

    console.log("Feature tables", gpkg.getFeatureTables());

    await gpkg
        .indexFeatureTable("edges", function (message) {
            console.log("Indexing", message);
        });
    const tableLayer = new L.GridLayer({noWrap: true, pane: 'overlayPane'});
    const featureDao = gpkg.getFeatureDao("edges");
    const ft = new FeatureTiles(featureDao, 512, 512);
    ft.maxFeaturesPerTile = 100000;
    ft.maxFeaturesTileDraw = new NumberFeaturesTile();

    tableLayer.createTile = function (tilePoint, done) {
        const ts = this.getTileSize();
        const canvas = L.DomUtil.create('canvas', 'leaflet-tile');
        canvas.width = 512;
        canvas.height = 512;
        if (!featureDao) return;

        drawTile(ft, tilePoint.x, tilePoint.y, tilePoint.z, ProjectionConstants.EPSG_3857, canvas, color_data).then(() => {
            console.log("Done");
            done(null, canvas);
        });
        return canvas;
    };

    if (previous_layer) {
        map.removeLayer(previous_layer);
    }
    previous_layer = tableLayer;
    map.addLayer(tableLayer);
}



L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
    opacity: 0.35,
}).addTo(map);

