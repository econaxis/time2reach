import mapboxgl from "mapbox-gl";

import { get_data, get_details } from "./get_data";

mapboxgl.accessToken = "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";
const map = new mapboxgl.Map({
    container: "map", // container ID
    style: "mapbox://styles/mapbox/dark-v11", // style URL
    center: [-79.43113401487446, 43.650685085905365], // starting position [lng, lat]
    zoom: 15 // starting zoom
});



map.on('load', async () => {
    let data_promise = await get_data({
        latitude: 43.70734532390574,
        longitude: -79.40832138061523
    })

    map.addSource("some id", {
        type: "vector",
        tiles: ["http://127.0.0.1:6767/edges/{z}/{x}/{y}.pbf"]
    });

    const layer = map.addLayer(
        {
            'id': 'somed', // Layer ID
            'type': 'line',
            'source': 'some id', // ID of the tile source created above
// Source has several layers. We visualize the one with name 'sequence'.
            'source-layer': 'edges',
            'layout': {
                'line-cap': 'round',
                'line-join': 'round'
            },
            'paint': {
                'line-opacity': 0.4,
                // 'line-color': 'rgb(173,0,0)',
                'line-color':
                    ['get', ["to-string", ['id']], ['literal', data_promise.m]],
                'line-width': 3.0
                // 'line-width': ['/', ['get', 'u'], 500000000]
            }
        },
    );

    map.on('dblclick', async (e) => {
        console.log('features', map.queryRenderedFeatures(e.point))
        e.preventDefault()
        data_promise = await get_data({
            latitude: e.lngLat.lat,
            longitude: e.lngLat.lng
        })
        map.setPaintProperty('somed', 'line-color', ['get', ["to-string", ['id']], ['literal', data_promise.m]])
    })

    const popup = new mapboxgl.Popup()
    map.on('mouseover', 'somed', async (e) => {
        const feature1 = map.queryRenderedFeatures(e.point)
        if(feature1.length) {
            popup.setLngLat(e.lngLat)
            const seconds = data_promise.raw[feature1[0].id]

            if(!seconds) return;

            const text = new Date(seconds * 1000).toISOString().substring(11, 19)
            popup.setText(text)
            popup.addTo(map)

            console.log(await get_details(data_promise, { latitude: e.lngLat.lat, longitude: e.lngLat.lng }))
        }
    })
    map.on('mouseleave', 'somed', (e) => {
        popup.remove()
    })

})

