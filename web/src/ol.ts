import mapboxgl from "mapbox-gl";

import { get_details } from "./get_data";
import { TimeColorMapper } from "./colors";

import "../tailwind.css";
import { format_popup_html, TripDetails } from "./format-details";

mapboxgl.accessToken = "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";
const map = new mapboxgl.Map({
    container: "map", // container ID
    style: "mapbox://styles/mapbox/dark-v11", // style URL
    center: [-79.43113401487446, 43.650685085905365], // starting position [lng, lat]
    zoom: 13 // starting zoom
});


map.on("load", async () => {

    let data_promise = await TimeColorMapper.fetch(new mapboxgl.LngLat(
        -79.61142287490227,
        43.68355164972115
    ));

    map.addSource("some id", {
        type: "vector",
        tiles: ["http://127.0.0.1:6767/edges/{z}/{x}/{y}.pbf"]
    });


    map.addLayer(
        {
            "id": "transit-layer", // Layer ID
            "type": "line",
            "source": "some id", // ID of the tile source created above
            "source-layer": "edges",
            "layout": {
                "line-cap": "round",
                "line-join": "round"
            },
            "paint": {
                "line-opacity": 0.4,
                "line-color":
                    ["get", ["to-string", ["id"]], ["literal", data_promise.m]],
                "line-width": 3.3
            }
        }
    );

    map.on("dblclick", async (e) => {
        console.log("features", map.queryRenderedFeatures(e.point));
        e.preventDefault();
        data_promise = await TimeColorMapper.fetch(e.lngLat);
        map.setPaintProperty("transit-layer", "line-color", ["get", ["to-string", ["id"]], ["literal", data_promise.m]]);
    });

    const popup = new mapboxgl.Popup({
        maxWidth: "none"
    });

    let currentTask = undefined;
    map.on("mouseover", "transit-layer", async (e) => {
        if (currentTask) clearTimeout(currentTask);

        map.getCanvas().style.cursor = "crosshair";
        currentTask = setTimeout(async () => {
            const feature1 = map.queryRenderedFeatures(e.point);

            if (feature1.length) {
                popup.setLngLat(e.lngLat);
                const seconds = data_promise.raw[feature1[0].id];

                if (!seconds) return;

                const details: Array<TripDetails> = await get_details(data_promise, {
                    latitude: e.lngLat.lat,
                    longitude: e.lngLat.lng
                });

                popup.setHTML(format_popup_html(seconds, details));
                popup.addTo(map);
            }
        }, 600);


    });
    map.on("mouseleave", "transit-layer", (e) => {
        map.getCanvas().style.cursor = "";
        popup.remove();
        clearTimeout(currentTask);
        currentTask = undefined;
    });

});

