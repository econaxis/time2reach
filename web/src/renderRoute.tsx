import React, { Fragment, useCallback, useEffect, useReducer } from "react";
import type mapboxgl from "mapbox-gl";
import { useQuery } from "react-query";
import { EMPTY_GEOJSON } from "./mapbox-map";
import RouteHighlight, {
    type HighlightedPointElev,
    type HighlightedPointGeoJSON,
} from "@/routeHighlight";
import { type Feature, type FeatureCollection, type LineString } from "geojson";
import { Settings } from "@/Settings";
import { baseUrl } from "@/dev-api";

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
    console.log("Rendering route", routeData)

    useEffect(() => {
        if (!map) {
            return;
        }

        if (!map.getSource("route")) {
            console.log("LOADING map");
            map.addSource("route", {
                type: "geojson",
                data: EMPTY_GEOJSON,
            });

            const COLORS = [
                '#44ce1b',
                '#89d72b',
                '#bbdb44',
                '#f7e379',
                '#f2a134',
                '#442608'
            ]

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
                        "line-color": ['match', ['get', 'bikeFriendly'],
0, COLORS[4],
1, COLORS[3],
2, COLORS[2],
3, COLORS[1],
4, COLORS[0],
5, COLORS[0],
COLORS[3]
                            ],
                        "line-width": 4.2,
                    },
                },
                // "admin1"
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
    const url = baseUrl + "/bike"
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
    setRouteMetadata: (meta: number[][], commit: boolean) => void
    setHighlightedPoints: (_: HighlightedPointElev) => void
    reverseOrgDest: () => void
}

export interface RouteSettings {
    avoidHills: number
    preferProtectedLanes: number
    commit: boolean
}

function defaultRouteSettings(): RouteSettings {
    return {
        avoidHills: 0.5,
        preferProtectedLanes: 0.5,
        commit: true
    };
}

function routeSettingsReducer(
    state: RouteSettings,
    action: { type: "setAvoidHills" | "setPreferProtectedLanes", value: number, commit: boolean }
): RouteSettings {
    switch (action.type) {
        case "setAvoidHills":
            return {
                ...state,
                avoidHills: action.value,
                commit: action.commit,
            };
        case "setPreferProtectedLanes":
            return {
                ...state,
                preferProtectedLanes: action.value,
                commit: action.commit,
            };
        default:
            return state;
    }
}

export interface EnergyResponse {
    calories: number
    uphill_meters: number
    downhill_meters: number
    total_meters: number
}

function splitRouteIntoSegments(route: GeoJSON.Feature<GeoJSON.LineString>, bikeFriendly: number[]): FeatureCollection<LineString> {
    const bikeFriendlyFeatures: Array<Feature<LineString>> = [];

    for (let i = 0; i < route.geometry.coordinates.length - 1; i++) {
        const lineStringFeature: Feature<LineString> = {
            type: "Feature",
            properties: {
                bikeFriendly: bikeFriendly[i + 1]
            },
            geometry: {
                type: "LineString",
                coordinates: [
                    route.geometry.coordinates[i],
                    route.geometry.coordinates[i + 1]
                ]
            }
        };

        bikeFriendlyFeatures.push(lineStringFeature);
    }

    return {
        type: "FeatureCollection",
        features: [
            ...bikeFriendlyFeatures,
        ]
    }
}

function defaultEnergy(): EnergyResponse {
    return {
        calories: 0,
        uphill_meters: 0,
        downhill_meters: 0,
        total_meters: 0
    };
}
function RenderBikeRoute_(props: RenderBikeRouteProps) {
    const { origin, destination, reverseOrgDest } = props;
    const [bikeFriendly, setBikeFriendly] = React.useState<number[]>([]);
    const prevEnergy = React.useRef<EnergyResponse>(defaultEnergy());

    const [routeSettings, dispatchRouteSettings] = useReducer(
        routeSettingsReducer,
        defaultRouteSettings()
    );

    const setAvoidHills = useCallback((x: number, commit: boolean) => {
        dispatchRouteSettings({ type: "setAvoidHills", value: x, commit });
    }, [])

    const setPreferProtectedLanes = useCallback((x: number, commit: boolean) => {
        dispatchRouteSettings({ type: "setPreferProtectedLanes", value: x, commit });
    }, [])

    const enabled = !!(origin && destination);

    const { data, isLoading, isError } = useQuery(
        ["bike-route", origin, destination, routeSettings],
        async() => {
            return await fetchBikeRoute(origin, destination, routeSettings);
        },
        {
            enabled,
        }
    );

    let energy: EnergyResponse = prevEnergy.current;
    let bikeRouteComponent: React.JSX.Element | null = null;

    useEffect(() => {
        if (data) {
            const { route_metadata: routeMetadata } = data;
            props.setRouteMetadata(routeMetadata, routeSettings.commit);
            setBikeFriendly(
                routeMetadata.map((a) => {
                    return a[2]
                })
            );
        }
    }, [data]);
    if (isError) {
        console.error("Error fetching bike route", data);
    } else if (isLoading || !data) {
        console.log("Loading bike route...")
    } else {
        const { route, elevation_index: elevationIndex, energy: energy_ } = data;

        energy = energy_ as EnergyResponse;
        prevEnergy.current = energy;

        const routeData: GeoJSON.FeatureCollection<LineString> = {
            type: "FeatureCollection",
            features: [route],
        };

        // Process routeData according to bike friendliness
        const renderedRoute = splitRouteIntoSegments(route, bikeFriendly);

        const setHighlightedPoints = (hp: HighlightedPointGeoJSON) => {
            // Map the index from the route to the elevation index
            const idx = elevationIndex[hp.geojson_index];
            props.setHighlightedPoints({ elevation_index: idx });
        };

        bikeRouteComponent = <RenderRoute map={props.map} routeData={renderedRoute}>
                    <RouteHighlight
                        map={props.map}
                        routeData={routeData}
                        setHighlightedPoints={setHighlightedPoints}
                    />
                </RenderRoute>;
    }

    return <>
        <Settings
            energy={energy}
            setAvoidHills={setAvoidHills}
            setPreferProtectedLanes={setPreferProtectedLanes}
            reverseOrgDest={reverseOrgDest}
        />
        {bikeRouteComponent}
    </>
}

export const RenderBikeRoute = React.memo(RenderBikeRoute_)
