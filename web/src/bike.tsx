import React, { useCallback, useRef, useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"
import { MapboxWrapper } from "@/mapbox-wrapper";
import { type HighlightedPointElev } from "@/routeHighlight";
import { ToFromSearchBox } from "@/search-box";

export interface OrgDest {
    origin: mapboxgl.LngLat
    destination: mapboxgl.LngLat
}

const DEFAULT_ORGDEST = {
    origin: new mapboxgl.LngLat(-122.480, 37.732),
    destination: new mapboxgl.LngLat(-122.4194, 37.7749)
    // destination: undefined
}

export type ElevData = number[][];
export interface ElevationChartData {
    background?: ElevData
    foreground: ElevData
    maxElevation: number
    maxDistance: number
}
function useHistory() {
    const [data, setData] = useState<ElevationChartData | undefined>(undefined);

    const lastForeground = useRef<ElevData | undefined>(undefined);
    const pushHistory = (elevationData: ElevData, commit: boolean) => {
        const maxElevation = Math.max(...elevationData.map(a => a[1]), data?.maxElevation ?? 0)
        const maxDistance = Math.max(...elevationData.map(a => a[0]), data?.maxDistance ?? 0)
        if (!commit) {
            if (!lastForeground.current) {
                lastForeground.current = data?.foreground;
            }
            setData((_data) => {
               return {
                     background: lastForeground.current,
                     foreground: elevationData,
                     maxElevation,
                   maxDistance
               }
            })
        } else {
            setData({
                    background: lastForeground.current,
                    foreground: elevationData,
                    maxElevation,
                maxDistance
            })
            lastForeground.current = undefined;
        }
    }

    const reset = () => {
        setData(undefined);
    }
    return { current: data, pushHistory, reset };
}

export function BikeMap() {
    const [queryClient] = React.useState(() => new QueryClient());

    const [orgDest, setOrgDest] = useState<OrgDest>(DEFAULT_ORGDEST); // [origin, destination
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);
    const { current, pushHistory, reset } = useHistory();
    const [highlightedPoint, setHighlightedPoint] = useState<HighlightedPointElev | undefined>(undefined); // [origin, destination

    const setOrgDestResetHistory = (...x: Parameters<typeof setOrgDest>) => {
        setOrgDest(...x);
        reset();
    };

    const mapOnLoad = (map: mapboxgl.Map) => {
        setMap(map);
    };

    let renderRouteMap: mapboxgl.Map | undefined;
    if (map != null) {
        renderRouteMap = map;
    }

    const setHighlightedPoints = useCallback((hp: HighlightedPointElev) => {
        setHighlightedPoint(hp)
    }, []);

    const reverseOrgDest = useCallback(() => {
        if (orgDest != null) {
            setOrgDestResetHistory({
                origin: orgDest.destination,
                destination: orgDest.origin
            })
        }
    }, [orgDest]);

    const setRouteMetadata = (routeMetadata: number[][], commit: boolean) => {
        if (routeMetadata) {
            const elevationData = routeMetadata.map(a => [a[0], a[1]])
            pushHistory(elevationData, commit);
        }
    }

    return (
        <QueryClientProvider client={queryClient}>
            <MapboxWrapper currentPos={new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}/>
                <ToFromSearchBox currentLocation={orgDest.origin} handleSetLocation={setOrgDestResetHistory} />
                <SetupMapbox setOrgDest={setOrgDestResetHistory} map={map} />
                <RenderBikeRoute reverseOrgDest={reverseOrgDest} origin={orgDest.origin} destination={orgDest.destination} map={renderRouteMap} setRouteMetadata={setRouteMetadata} setHighlightedPoints={setHighlightedPoints}>
                    <ElevationChart elevationData={current} highlightedPoint={highlightedPoint} className={''}/>
                </RenderBikeRoute>
        </QueryClientProvider>
    );
}
