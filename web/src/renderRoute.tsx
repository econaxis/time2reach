import { Fragment, useEffect } from "react";
import type mapboxgl from "mapbox-gl";
import { useQuery } from "react-query";
import { EMPTY_GEOJSON } from "./mapbox-map";

export interface RenderStraightRouteProps {
    map: mapboxgl.Map | undefined
    origin: mapboxgl.LngLat | undefined
    destination: mapboxgl.LngLat | undefined
}

export interface RenderRouteProps {
    map: mapboxgl.Map | undefined
    routeData?: GeoJSON.FeatureCollection
}

export function RenderRoute(props: RenderRouteProps) {
    const { map, routeData } = props;

    useEffect(() => {
        if (!map) {
            return;
        }

        console.log("LOADING map");

        if (!map.getSource("route")) {
            map.addSource("route", {
                type: "geojson",
                data: EMPTY_GEOJSON
            });

            map.addLayer({
                id: "route",
                type: "line",
                source: "route",
                layout: {
                    "line-join": "round",
                    "line-cap": "round",
                    "line-sort-key": 1000
                },
                paint: {
                    "line-color": "#445fb0",
                    "line-width": 4.2,
                }
            }, "admin1");
        }
    }, [map]);
    useEffect(() => {
        if (!map || !routeData) {
            return;
        }

        (map.getSource("route") as mapboxgl.GeoJSONSource).setData(routeData);
    }, [map, routeData]);

    return <Fragment />;
}

export function RenderStraightRoute(props: RenderStraightRouteProps) {
    const { origin, destination } = props;

    if (!origin || !destination) {
        return <Fragment />;
    }

    const routeData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: [
            {
                type: "Feature",
                properties: {},
                geometry: {
                    type: "LineString",
                    coordinates: [
                        [origin.lng, origin.lat],
                        [destination.lng, destination.lat]
                    ]
                }
            }
        ]
    };
    return <RenderRoute map={props.map} routeData={routeData} />;
}

async function fetchBikeRoute(origin?: mapboxgl.LngLat, destination?: mapboxgl.LngLat) {
    if (!origin || !destination) {
        throw new Error("Origin or destination not set");
    }
    const url = `http://localhost:3030/bike`;
    const postData = {
        start: {
            latitude: origin.lat,
            longitude: origin.lng
        },
        end: {
            latitude: destination.lat,
            longitude: destination.lng
        }
    };
    const req = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(postData)
    });

    if (!req.ok) {
        throw new Error(`Failed to fetch bike route: ${req.status}  ${await req.text()}`);
    }
    return await req.json();
}

export interface RenderBikeRouteProps extends RenderStraightRouteProps {
    setElevations: (elevations: number[]) => void
}

export function RenderBikeRoute(props: RenderBikeRouteProps) {
    const { origin, destination } = props;

    const enabled = !!(origin && destination);

    // Use react-query to query the bike route
    const { data, isLoading } = useQuery(["bike-route", origin, destination], async() => {
        return await fetchBikeRoute(origin, destination);
    }, {
        enabled
    });

    if (isLoading || !data) {
        return <Fragment />;
    }

    const { route, elevation } = data;

    const routeData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: [
            route
        ]
    };

    props.setElevations(elevation);
    return <RenderRoute map={props.map} routeData={routeData} />;
}
