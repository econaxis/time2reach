import React, { useCallback, useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"
import { MapboxWrapper } from "@/mapbox-wrapper";
import { type HighlightedPointElev } from "@/routeHighlight";

export interface OrgDest {
    origin: mapboxgl.LngLat
    destination: mapboxgl.LngLat
}

const DEFAULT_ORGDEST = {
    origin: new mapboxgl.LngLat(-122.450, 37.782),
    destination: new mapboxgl.LngLat(-122.4194, 37.7749)
    // destination: undefined
}

export function hashElevationData(elevationData: number[][]): string {
    let number = 0;

    let index = 0;
    for (const elev of elevationData) {
        index += 0.2;
        number += elev[0] * index + elev[1] * index;
    }

    return number.toString() + elevationData.length.toString();
}

function swapXY<T>(a: T[], idx1: number, idx2: number) {
    const tmp = a[idx1];
    a[idx1] = a[idx2];
    a[idx2] = tmp;
}

type ElevData = number[][];
export interface ElevationChartData {
    background: ElevData
    foreground: ElevData
}
function useHistory() {
    const [history1, setHistory1] = useState<number[][][]>([]);
    const [currentNotCommit, setCurrentNotCommit] = useState<number[][] | undefined>(undefined);

    const pushHistory = useCallback((elevationData: number[][], commit: boolean) => {
        console.log("Commit", commit)
        if (!commit) {
            setCurrentNotCommit(elevationData);
            return
        } else {
            setCurrentNotCommit(undefined);
        }
        setHistory1((history_) => {
            const history = [...history_];

            const hash = hashElevationData(elevationData);
            const foundIndex = history.findIndex((a) => hashElevationData(a) === hash)
            if (foundIndex !== -1) {
                if (history.length > 2) {
                    // Always make sure the *last* route is the second last in history position
                    swapXY(history, history.length - 2, history.length - 1)
                }
                swapXY(history, foundIndex, history.length - 1)
                return history;
            }
            history.push(elevationData);
            return history;
        });
    }, [])

    const reset = useCallback(() => {
        setHistory1([]);
    }, []);

    let current: number[][];
    if (currentNotCommit) {
        current = currentNotCommit;
    } else {
        current = history1[history1.length - 1];
    }

    return { current, history: history1, pushHistory, reset };
}

export function BikeMap() {
    const [queryClient] = React.useState(() => new QueryClient());

    const [orgDest, setOrgDest] = useState<OrgDest>(DEFAULT_ORGDEST); // [origin, destination
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);
    const { current, history, pushHistory, reset } = useHistory();
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
                <ElevationChart elevationData={current} elevationDataHistory={history} hp={highlightedPoint}/>
                <SetupMapbox setOrgDest={setOrgDestResetHistory} map={map} />
                <RenderBikeRoute reverseOrgDest={reverseOrgDest} origin={orgDest.origin} destination={orgDest.destination} map={renderRouteMap} setRouteMetadata={setRouteMetadata} setHighlightedPoints={setHighlightedPoints}/>
        </QueryClientProvider>
    );
}
