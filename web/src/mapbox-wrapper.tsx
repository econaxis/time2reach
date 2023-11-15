import { useEffect, useRef, useState } from "react";
import mapboxgl from "mapbox-gl";

export interface Props {
    currentPos: mapboxgl.LngLat
    onLoad: (map: mapboxgl.Map) => void
    children: React.ReactNode
}

export function MapboxWrapper(props: Props) {
    const currentPos = props.currentPos;
    const [map, setMap] = useState<mapboxgl.Map | null>(null);
    // const [mapboxLoading, setMapboxLoading] = useState(true);
    const mapContainer = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        // Init mapbox gl map here.
        if (mapContainer.current == null) return;
        if (map !== null) return;

        console.log("<MapboxWrapper> LOADING MAP!");

        mapboxgl.accessToken =
            "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";

        const map1 = new mapboxgl.Map({
            container: mapContainer.current, // container ID
            style: "mapbox://styles/mapbox/outdoors-v12", // style URL
            center: currentPos, // starting position [lng, lat]
            zoom: 13, // starting zoom
            preserveDrawingBuffer: true,
        });
        setMap(map1);
        map1.doubleClickZoom.disable();
        map1.on("load", () => {
            map1.addSource("mapbox-streets", {
                type: "vector",
                url: "mapbox://mapbox.mapbox-streets-v8",
            });
            map1.addLayer({
                id: "admin1",
                source: "mapbox-streets",
                "source-layer": "road",
                type: "line",
                layout: {
                    "line-sort-key": 1
                },
                paint: {
                    "line-color": '#56be43',
                    "line-width": 1.6,
                    "line-dasharray": [
                        "match",
                        ["get", "bike_lane"],
                        "both",
                        ["literal", []],
                        "yes",
                        ["literal", [1]],
                        "left",
                        ["literal", []],
                        "right",
                        ["literal", []],
                        ["literal", []],
                    ]
                },
                filter: [
                    "match",
                    ["get", "bike_lane"],
                    "no",
                    false,
                    "both",
                    true,
                    "yes",
                    true,
                    "left",
                    true,
                    "right",
                    true,
                    false,
                ],
            });
            props.onLoad(map1);
        });
    }, []);

    return (
        <div ref={mapContainer} className="map w-screen h-screen overflow-none">
            {props.children}
        </div>
    );
}
