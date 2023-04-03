import mapboxgl from "mapbox-gl";

import { get_details } from "./get_data";
import { TimeColorMapper } from "./colors";

mapboxgl.accessToken = "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";
const map = new mapboxgl.Map({
    container: "map", // container ID
    style: "mapbox://styles/mapbox/dark-v11", // style URL
    center: [-79.43113401487446, 43.650685085905365], // starting position [lng, lat]
    zoom: 15 // starting zoom
});

function format_popup_html(arrival_time: string, details: string) {
    return `<strong>${arrival_time}</strong><p>${details}</p>`;
}


map.on("load", async () => {

    let data_promise = await TimeColorMapper.fetch(new mapboxgl.LngLat(
        -79.40832138061523,
        43.70734532390574
    ));

    map.addSource("some id", {
        type: "vector",
        tiles: ["http://127.0.0.1:6767/edges/{z}/{x}/{y}.pbf"]
    });


    const layer = map.addLayer(
        {
            "id": "somed", // Layer ID
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
        map.setPaintProperty("somed", "line-color", ["get", ["to-string", ["id"]], ["literal", data_promise.m]]);
    });

    const popup = new mapboxgl.Popup({
        className: "popup"
    });

    let currentTask = undefined;
    map.on("mouseover", "somed", async (e) => {
        if (currentTask) clearTimeout(currentTask);

        map.getCanvas().style.cursor = "crosshair";
        currentTask = setTimeout(async () => {
            const feature1 = map.queryRenderedFeatures(e.point);

            if (feature1.length) {
                popup.setLngLat(e.lngLat);
                const seconds = data_promise.raw[feature1[0].id];

                if (!seconds) return;

                const arrival_time = new Date(seconds * 1000).toISOString().substring(11, 19);
                const detail_text = await get_details(data_promise, {
                    latitude: e.lngLat.lat,
                    longitude: e.lngLat.lng
                });

                popup.setHTML(format_popup_html(arrival_time, detail_text));
                popup.addTo(map);
            }
        }, 600);


    });
    map.on("mouseleave", "somed", (e) => {
        map.getCanvas().style.cursor = "";
        popup.remove();
        clearTimeout(currentTask);
        currentTask = undefined;
    });

});

