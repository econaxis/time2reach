import { Fragment, useEffect } from "react";
import type mapboxgl from "mapbox-gl";
import { useQuery } from "react-query";
import { EMPTY_GEOJSON } from "./mapbox-map";
import RouteHighlight, { HighlightedPointElev, type HighlightedPointGeoJSON } from "@/routeHighlight";
import { type LineString } from "geojson";

export interface RenderStraightRouteProps {
    map: mapboxgl.Map | undefined
    origin: mapboxgl.LngLat | undefined
    destination: mapboxgl.LngLat | undefined
}

export interface RenderRouteProps {
    map: mapboxgl.Map | undefined
    routeData?: GeoJSON.FeatureCollection<LineString>
    children?: React.ReactNode
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

    return <Fragment> {props.children} </Fragment>;
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
    setHighlightedPoints: (_: HighlightedPointElev) => void
}

export function RenderBikeRoute(props: RenderBikeRouteProps) {
    const { origin, destination } = props;

    const enabled = !!(origin && destination);

    // Use react-query to query the bike route
    const { data, isLoading, isError } = useQuery(["bike-route", origin, destination], async() => {
        return await fetchBikeRoute(origin, destination);
    }, {
        enabled
    });

    if (isError) {
        console.error("Error fetching bike route", data);
        return <Fragment />;
    }
    if (isLoading || !data) {
        return <Fragment />;
    }

    const { route, elevation, elevation_index: elevationIndex } = data;

    const routeData: GeoJSON.FeatureCollection<LineString> = {
        type: "FeatureCollection",
        features: [
            route
        ]
    };

    const setHighlightedPoints = (hp: HighlightedPointGeoJSON) => {
        // Map the index from the route to the elevation index
        const idx = elevationIndex[hp.geojson_index];
        props.setHighlightedPoints({ elevation_index: idx })
    }

    props.setElevations(elevation);
    return <RenderRoute map={props.map} routeData={routeData}>
        <RouteHighlight map={props.map} routeData={routeData} setHighlightedPoints={setHighlightedPoints}/>
    </RenderRoute>;
}
