import { useEffect, useRef, useState } from "react";
import mapboxgl from "mapbox-gl";

export interface Props {
    currentPos: mapboxgl.LngLat
    onLoad: (map: mapboxgl.Map) => void
    children?: React.ReactNode
}

let mapInitialized = false;

export function MapboxWrapper(props: Props) {
    const currentPos = props.currentPos;
    const [map, setMap] = useState<mapboxgl.Map | null>(null);
    const mapContainer = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        // Init mapbox gl map here.
        if (mapContainer.current == null) return;
        if (map !== null) return;
        if (mapInitialized) return;
        mapInitialized = true;

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
            map1.addSource("mapbox-terrain", {
                type: "vector",
                url: "mapbox://mapbox.mapbox-terrain-v2"
            });

            if (!map1.getLayer("hillshade1")) {
                console.log("Adding hillshade")
                // map1.addLayer({
                //     id: "hillshade1",
                //     source: "mapbox-terrain",
                //     "source-layer": "hillshade",
                //     type: "fill",
                //     paint: {
                //         "fill-color": "rgba(66,100,251, 0.3)",
                //         "fill-outline-color": "rgba(66,100,251, 1)",
                //     },
                // });

                // map1.addLayer({
                //     id: "contour1",
                //     source: "mapbox-terrain",
                //     "source-layer": "contour",
                //     type: "line",
                //     paint: {
                //         "line-color": "#505050"
                //     }
                // })

                map1.addSource('dem', {
                    type: 'raster-dem',
                    url: 'mapbox://mapbox.mapbox-terrain-dem-v1'
                });
                map1.setTerrain({ source: 'dem', exaggeration: 5 });
                map1.addLayer(
                    {
                        id: 'hillshading',
                        source: 'dem',
                        type: 'hillshade'
                    },
                    'land-structure-polygon'
                );
            }
            // map1.addLayer({
            //     id: "admin1",
            //     source: "mapbox-streets",
            //     "source-layer": "road",
            //     type: "line",
            //     layout: {
            //         "line-sort-key": 1
            //     },
            //     paint: {
            //         "line-color": '#56be43',
            //         "line-width": 1.6,
            //     },
            // });
            props.onLoad(map1);
        });
    }, []);

    return <div ref={mapContainer} className="map w-screen h-screen overflow-none"></div>;
}
