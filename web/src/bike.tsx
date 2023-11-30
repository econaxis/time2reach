import React, { useEffect, useReducer, useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"
import { MapboxWrapper } from "@/mapbox-wrapper";
import { type HighlightedPointElev, HighlightedPointGeoJSON, useRouteHighlight } from "@/routeHighlight";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Line } from "react-chartjs-2";
import Settings from "@/Settings";

export interface OrgDest {
    origin: mapboxgl.LngLat
    destination: mapboxgl.LngLat
}

const DEFAULT_ORGDEST = {
    origin: new mapboxgl.LngLat(-122.4194, 37.7749),
    destination: new mapboxgl.LngLat(-122.4194, 37.7749)
}

export interface RouteSettings {
    avoidHills: number
    preferProtectedLanes: number
}

function defaultRouteSettings(): RouteSettings {
    return {
        avoidHills: 0.5,
        preferProtectedLanes: 0.5
    }
}

function routeSettingsReducer(state: RouteSettings, action: { type: "setAvoidHills" | "setPreferProtectedLanes", value: number }): RouteSettings {
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
export function BikeMap() {
    const queryClient = new QueryClient({});

    const [orgDest, setOrgDest] = useState<OrgDest | undefined>(DEFAULT_ORGDEST); // [origin, destination
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);
    const [elevations, setElevations] = useState<number[] | undefined>(undefined);
    const [highlightedPoint, setHighlightedPoint] = useState<HighlightedPointElev | undefined>(undefined); // [origin, destination

    const [routeSettings, dispatchRouteSettings] = useReducer(routeSettingsReducer, defaultRouteSettings());

    const mapOnLoad = (map: mapboxgl.Map) => {
        setMap(map);
    };

    let renderRouteMap: mapboxgl.Map | undefined;
    if (map != null) {
        renderRouteMap = map;
    }

    const setHighlightedPoints = (hp: HighlightedPointElev) => {
        setHighlightedPoint(hp)
    };

    return (
        <QueryClientProvider client={queryClient}>
            <MapboxWrapper currentPos={new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}>
                <ElevationChart elevationData={elevations} hp={highlightedPoint}/>
                <Settings setAvoidHills={x => { dispatchRouteSettings({ type: "setAvoidHills", value: x }); }} setPreferProtectedLanes={x => { dispatchRouteSettings({ type: "setPreferProtectedLanes", value: x }); }}/>
                <SetupMapbox setOrgDest={setOrgDest} map={map} />
                <RenderBikeRoute origin={orgDest.origin} destination={orgDest.destination} map={renderRouteMap} setElevations={setElevations} setHighlightedPoints={setHighlightedPoints} routeSettings={routeSettings}/>
            </MapboxWrapper>
        </QueryClientProvider>
    );
}
