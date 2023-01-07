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

const cmap = createColorMap({
    alpha: 1,
    colormap: 'jet',
    format: 'hex',
    nshades: 100
});

window.cmap = cmap;

const tileCache = new Map();

window.tileCache = tileCache;

function gen_key(x, y, z) {
    return `${x}_${y}_${z}`
}


async function drawTile(ft: FeatureTiles, x: number, y: number, z: number, tileProjection, tileCanvas?: any) {
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
    console.log(boundingBox.width, boundingBox.height);
    const expandedBoundingBox = ft.expandBoundingBox(boundingBox, tileProjection);


    let zoom = map.getZoom();

    let modulo_cutoff = 1;

    if (zoom >= 12) {
        modulo_cutoff = 1;
    } else if (zoom >= 11) {
        modulo_cutoff = 3;
    } else if (zoom >= 10) {
        modulo_cutoff = 9;
    } else if (zoom >= 8) {
        modulo_cutoff = 18;
    }

    // get number of features that could intercept this bounding box
    const featureCount = ft.featureDao.countInBoundingBox(expandedBoundingBox, tileProjection);
    if (featureCount > 0) {
        if (ft.maxFeaturesPerTile == null || featureCount <= ft.maxFeaturesPerTile) {
            const transform = ft.getTransformFunction(tileProjection);
            const iterator = ft.featureDao.fastQueryBoundingBox(expandedBoundingBox, tileProjection);
            for (const featureRow of iterator) {
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

                    const time = featureRow.values?.["test_field1"];
                    if (time === null) {
                        ft.linePaint.color = "#A7727277"
                    } else {
                        let time_mapped = (time - 46810) / (49960 - 46810);
                        time_mapped = Math.round(time_mapped * 100);

                        time_mapped = Math.min(99, time_mapped);
                        time_mapped = Math.max(0, time_mapped);
                        ft.linePaint.color = cmap[time_mapped];
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

async function load_gpkg() {
    const gpkg = await GeoPackageAPI.open("/toronto2.gpkg");
    await Canvas.initializeAdapter();

    console.log("Feature tables", gpkg.getFeatureTables());

    await gpkg
        .indexFeatureTable("edges", function (message) {
            console.log("Indexing", message);
        });
    const tableLayer = new L.GridLayer({noWrap: true, pane: 'overlayPane'});
    const featureDao = gpkg.getFeatureDao("edges");
    const ft = new FeatureTiles(featureDao, 256, 256);
    ft.maxFeaturesPerTile = 100000;
    ft.maxFeaturesTileDraw = new NumberFeaturesTile();

    tableLayer.createTile = function (tilePoint, done) {
        const canvas = L.DomUtil.create('canvas', 'leaflet-tile');
        canvas.width = 256;
        canvas.height = 256;
        if (!featureDao) return;


        drawTile(ft, tilePoint.x, tilePoint.y, tilePoint.z, ProjectionConstants.EPSG_3857, canvas).then(() => {
            done(null, canvas);
        });



        return canvas;
    };
    map.addLayer(tableLayer);
}


load_gpkg();

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
    opacity: 0.4
}).addTo(map);

