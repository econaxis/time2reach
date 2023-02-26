import "./style.css";
import Geometry from "geojson";
import {
    Canvas,
    FeatureTiles,
    GeoPackageAPI,
    NumberFeaturesTile,
    ProjectionConstants,
    setSqljsWasmLocateFile,
    TileBoundingBoxUtils,
} from "@ngageoint/geopackage";
import L, {Control} from "leaflet";
import {CrsGeometry} from "@ngageoint/geopackage/dist/lib/types/CrsGeometry";
import {cmap, TimeColorMapper} from "./colors";
import Zoom = Control.Zoom;

setSqljsWasmLocateFile(
    (filename) =>
        `https://unpkg.com/@ngageoint/geopackage@4.2.3/dist/` + filename
);
const map = L.map("map").setView([43.657628, -79.450641], 13);
const gpkg = await GeoPackageAPI.open("/toronto2.gpkg");


const tileCache = new Map();

window.tileCache = tileCache;
window.map = map;

map.on("click", (evt) => {
    const body = {
        latitude: evt.latlng.lat,
        longitude: evt.latlng.lng,
    };
    tileCache.clear();
    console.log("Loading...");
    load_gpkg(body).then(() => {
        console.log("Done loading new coords", evt.latlng);
    });
});

load_gpkg({
    latitude: 43.70734532390574,
    longitude: -79.40832138061523
})

function gen_key(x, y, z) {
    return `${x}_${y}_${z}`;
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

function get_modulo_filter(zoom: number) {
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
    return modulo_cutoff;
}

async function drawTile(
    ft: FeatureTiles,
    x: number,
    y: number,
    z: number,
    tileCanvas: any,
    color_data: TimeColorMapper
) {
    const context = tileCanvas.getContext("2d");
    const width = ft.tileWidth;
    const height = ft.tileHeight;
    context.clearRect(0, 0, width, height);
    if (tileCache.get(gen_key(x, y, z))) {
        console.log("Using cache for", x, y, z);
        const imgdata = tileCache.get(gen_key(x, y, z));
        context.putImageData(imgdata, 0, 0);
        return;
    }

    const boundingBox = TileBoundingBoxUtils.getWebMercatorBoundingBoxFromXYZ(
        x,
        y,
        z
    );
    // console.log(x, y, z, boundingBox);
    const expandedBoundingBox = ft.expandBoundingBox(
        boundingBox,
        ProjectionConstants.EPSG_3857
    );


    let modulo_cutoff = get_modulo_filter(map.getZoom());

    // get number of features that could intercept this bounding box
    const featureCount = ft.featureDao.countInBoundingBox(
        expandedBoundingBox,
        ProjectionConstants.EPSG_3857
    );
    ft.linePaint.strokeWidth = 4;
    console.log("Feature count", featureCount)
    if (featureCount > 0) {
        if (
            ft.maxFeaturesPerTile == null ||
            featureCount <= ft.maxFeaturesPerTile
        ) {
            const transform = ft.getTransformFunction(
                ProjectionConstants.EPSG_3857
            );
            const iterator = ft.featureDao.fastQueryBoundingBox(
                expandedBoundingBox,
                ProjectionConstants.EPSG_3857
            );
            for (const featureRow of iterator) {
                if (Math.random() < 0.01) {
                    await sleep(10);
                }
                if (
                    (featureRow.values?.["highway"] === "service" ||
                        featureRow.values?.["highway"] === "residential") &&
                    featureRow.values?.["u"] % modulo_cutoff !== 0
                ) {
                    continue;
                }
                if (featureRow.geometry != null) {
                    let geojson = null;
                    if (ft.cacheGeometries) {
                        geojson = ft.geometryCache.getGeometry(featureRow.id);
                    }
                    if (!geojson) {
                        geojson =
                            featureRow.geometry.geometry.toGeoJSON() as Geometry &
                                CrsGeometry;
                        ft.geometryCache.setGeometry(featureRow.id, geojson);
                    }
                    const style = ft.getFeatureStyle(featureRow);

                    const from_node = <number>featureRow.values?.["from"];
                    const to_node = <number>featureRow.values?.["to"];

                    ft.linePaint.color = color_data.get_color(from_node, to_node);
                    //
                    // console.log("Drawing geometry")
                    await ft.drawGeometry(
                        geojson,
                        context,
                        boundingBox,
                        style,
                        transform
                    );
                }
            }
        }
    }

    tileCache.set(gen_key(x, y, z), context.getImageData(0, 0, width, height));
}

async function get_data(body): Promise<TimeColorMapper> {
    const data = await fetch("http://localhost:3030/hello", {
        method: "POST",
        mode: "cors",
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    });
    const js = await data.json();

    const colors = new TimeColorMapper();
    colors.m = js;
    for (const nodeid in js) {
        const time = js[nodeid];
        colors.min = Math.min(colors.min, time);
        colors.max = Math.max(colors.max, time);
    }
    return colors;
}

let previous_layer = null;

const TILESIZE = 512;

async function load_gpkg(body) {
    const color_data = await get_data(body);
    await Canvas.initializeAdapter();

    console.log("Feature tables", gpkg.getFeatureTables());

    await gpkg.indexFeatureTable("edges", function (message) {
        console.log("Indexing", message);
    });
    const tableLayer = new L.GridLayer({noWrap: true, pane: "overlayPane", tileSize: TILESIZE});
    const featureDao = gpkg.getFeatureDao("edges");
    const ft = new FeatureTiles(featureDao, TILESIZE, TILESIZE);
    ft.maxFeaturesPerTile = 100000;
    ft.maxFeaturesTileDraw = new NumberFeaturesTile();

    tableLayer.createTile = function (tilePoint, done) {
        const canvas = L.DomUtil.create("canvas", "leaflet-tile");
        canvas.width = TILESIZE;
        canvas.height = TILESIZE;
        if (!featureDao) return;

        let zoom_diff;
        if (TILESIZE == 512) zoom_diff = 1
        else if (TILESIZE == 1024) zoom_diff = 2
        else if (TILESIZE == 256) zoom_diff = 0

        drawTile(
            ft,
            tilePoint.x,
            tilePoint.y,
            tilePoint.z - zoom_diff,
            canvas,
            color_data
        ).then(() => {
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

L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution:
        '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
    opacity: 0.35,
}).addTo(map);
