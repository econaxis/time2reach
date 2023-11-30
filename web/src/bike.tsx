import React, { useEffect, useReducer, useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"
import { MapboxWrapper } from "@/mapbox-wrapper";
import { type HighlightedPointElev, HighlightedPointGeoJSON, useRouteHighlight } from "@/routeHighlight";

export interface OrgDest {
    origin: mapboxgl.LngLat
    destination: mapboxgl.LngLat
}

const DEFAULT_ORGDEST = {
    origin: new mapboxgl.LngLat(-122.450, 37.782),
    destination: new mapboxgl.LngLat(-122.4194, 37.7749)
}


export function BikeMap() {
    const queryClient = new QueryClient({});

    const [orgDest, setOrgDest] = useState<OrgDest | undefined>(DEFAULT_ORGDEST); // [origin, destination
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);
    const [elevations, setElevations] = useState<number[] | undefined>(undefined);
    const [highlightedPoint, setHighlightedPoint] = useState<HighlightedPointElev | undefined>(undefined); // [origin, destination

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

    const reverseOrgDest = () => {
        if (orgDest != null) {
            setOrgDest({
                origin: orgDest.destination,
                destination: orgDest.origin
            })
        }
    }

    return (
        <QueryClientProvider client={queryClient}>
            <MapboxWrapper currentPos={new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}>
            </MapboxWrapper>
                <ElevationChart elevationData={elevations} hp={highlightedPoint}/>
                <SetupMapbox setOrgDest={setOrgDest} map={map} />
                <RenderBikeRoute reverseOrgDest={reverseOrgDest} origin={orgDest.origin} destination={orgDest.destination} map={renderRouteMap} setElevations={setElevations} setHighlightedPoints={setHighlightedPoints}/>
        </QueryClientProvider>
    );
}
