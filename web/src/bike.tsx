import { useState } from "react";
import mapboxgl from "mapbox-gl";
import { MapboxWrapper } from "./mapbox-wrapper";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";

export function BikeMap() {
    const queryClient = new QueryClient({});

    const [origin, setOrigin] = useState<mapboxgl.LngLat | undefined>(undefined);
    const [destination, setDestination] = useState<mapboxgl.LngLat | undefined>(undefined);
    const [map, setMap] = useState<mapboxgl.Map | undefined>(undefined);

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
                <SetupMapbox setOrigin={setOrigin} setDestination={setDestination} map={map} />
                <RenderBikeRoute origin={origin} destination={destination} map={renderRouteMap} />
            </MapboxWrapper>
        </QueryClientProvider>
    );
}
