import { useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"
import { MapboxWrapper } from "@/mapbox-wrapper";

export interface OrgDest {
    origin: mapboxgl.LngLat
    destination: mapboxgl.LngLat
}

const DEFAULT_ORGDEST = {
    origin: new mapboxgl.LngLat(-122.4194, 37.7749),
    destination: new mapboxgl.LngLat(-122.4194, 37.7749)
}

export function BikeMap() {
    const queryClient = new QueryClient({});

    const [orgDest, setOrgDest] = useState<OrgDest | undefined>(DEFAULT_ORGDEST); // [origin, destination
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);
    const [elevations, setElevations] = useState<number[] | undefined>(undefined);

    const mapOnLoad = (map: mapboxgl.Map) => {
        setMap(map);
    };

    let renderRouteMap: mapboxgl.Map | undefined;
    if (map != null) {
        renderRouteMap = map;
    }


    return (
        <QueryClientProvider client={queryClient}>
            <MapboxWrapper currentPos={new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}>
                <ElevationChart data={elevations}/>
                <SetupMapbox setOrgDest={setOrgDest} map={map} />
                <RenderBikeRoute origin={orgDest.origin} destination={orgDest.destination} map={renderRouteMap} setElevations={setElevations} />
            </MapboxWrapper>
        </QueryClientProvider>
    );
}
