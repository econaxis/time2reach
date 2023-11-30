import React, { Fragment, useCallback, useEffect, useReducer } from "react";
import type mapboxgl from "mapbox-gl";
import { useQuery } from "react-query";
import { EMPTY_GEOJSON } from "./mapbox-map";
import RouteHighlight, {
    type HighlightedPointElev,
    type HighlightedPointGeoJSON,
} from "@/routeHighlight";
import { type LineString } from "geojson";
import { Settings } from "@/Settings";

export const ROUTE_COLOR_BLUE = "#6A7EB8";

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
                data: EMPTY_GEOJSON,
            });

            map.addLayer(
                {
                    id: "route",
                    type: "line",
                    source: "route",
                    layout: {
                        "line-join": "round",
                        "line-cap": "round",
                        "line-sort-key": 1000,
                    },
                    paint: {
                        "line-color": ROUTE_COLOR_BLUE,
                        "line-width": 4.2,
                    },
                },
                "admin1"
            );
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

async function fetchBikeRoute(
    origin?: mapboxgl.LngLat,
    destination?: mapboxgl.LngLat,
    routeSettings?: RouteSettings
) {
    if (!origin || !destination || !routeSettings) {
        throw new Error("Origin or destination or route settings not set");
    }
    const url = `http://localhost:3030/bike`;
    const postData = {
        start: {
            latitude: origin.lat,
            longitude: origin.lng,
        },
        end: {
            latitude: destination.lat,
            longitude: destination.lng,
        },
        options: routeSettings,
    };
    const req = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(postData),
    });

    if (!req.ok) {
        throw new Error(`Failed to fetch bike route: ${req.status}  ${await req.text()}`);
    }
    return await req.json();
}

export interface RenderBikeRouteProps extends RenderStraightRouteProps {
    setElevations: (elevations: number[]) => void
    setHighlightedPoints: (_: HighlightedPointElev) => void
    reverseOrgDest: () => void
}

export interface RouteSettings {
    avoidHills: number
    preferProtectedLanes: number
}

function defaultRouteSettings(): RouteSettings {
    return {
        avoidHills: 0.5,
        preferProtectedLanes: 0.5,
    };
}

function routeSettingsReducer(
    state: RouteSettings,
    action: { type: "setAvoidHills" | "setPreferProtectedLanes", value: number }
): RouteSettings {
    switch (action.type) {
        case "setAvoidHills":
            return {
                ...state,
                avoidHills: action.value,
            };
        case "setPreferProtectedLanes":
            return {
                ...state,
                preferProtectedLanes: action.value,
            };
        default:
            return state;
    }
}

export interface EnergyResponse {
    calories: number
    uphill_meters: number
    downhill_meters: number
}

export function RenderBikeRoute(props: RenderBikeRouteProps) {
    const { origin, destination, reverseOrgDest } = props;
    const previousCalories = React.useRef<number>(0);

    const [routeSettings, dispatchRouteSettings] = useReducer(
        routeSettingsReducer,
        defaultRouteSettings()
    );

    const setAvoidHills = useCallback((x) => {
        dispatchRouteSettings({ type: "setAvoidHills", value: x });
    }, [])

    const setPreferProtectedLanes = useCallback((x) => {
        dispatchRouteSettings({ type: "setPreferProtectedLanes", value: x });
    }, [])

    const enabled = !!(origin && destination);

    // Use react-query to query the bike route
    const { data, isLoading, isError } = useQuery(
        ["bike-route", origin, destination, routeSettings],
        async() => {
            return await fetchBikeRoute(origin, destination, routeSettings);
        },
        {
            enabled,
        }
    );

    let calories: number = previousCalories.current;
    let bikeRouteComponent: React.JSX.Element = null;
    if (isError) {
        console.error("Error fetching bike route", data);
    } else if (isLoading || !data) {
        console.log("Loading bike route...")
    } else {
        const { route, elevation, elevation_index: elevationIndex, energy } = data;

        calories = energy.calories;
        previousCalories.current = calories;

        const routeData: GeoJSON.FeatureCollection<LineString> = {
            type: "FeatureCollection",
            features: [route],
        };

        const setHighlightedPoints = (hp: HighlightedPointGeoJSON) => {
            // Map the index from the route to the elevation index
            const idx = elevationIndex[hp.geojson_index];
            props.setHighlightedPoints({ elevation_index: idx });
        };

        props.setElevations(elevation);
        bikeRouteComponent = <RenderRoute map={props.map} routeData={routeData}>
                    <RouteHighlight
                        map={props.map}
                        routeData={routeData}
                        setHighlightedPoints={setHighlightedPoints}
                    />
                </RenderRoute>;
    }

    return <>
        <Settings
            calories={calories} // TODO: replace with previous calories
            setAvoidHills={setAvoidHills}
            setPreferProtectedLanes={setPreferProtectedLanes}
            reverseOrgDest={reverseOrgDest}
        />
        {bikeRouteComponent}
    </>
}
