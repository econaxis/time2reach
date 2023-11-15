import { useState } from "react";
import mapboxgl from "mapbox-gl";
import { QueryClient, QueryClientProvider } from "react-query";
import { SetupMapbox } from "./setupMapbox";
import { RenderBikeRoute } from "./renderRoute";
import ElevationChart from "./elevation-chart";
import "../app/globals.css"

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

    const numbersArray = Array.from({ length: 100 }, (_, i) =>
        Math.floor(Math.random() * (i * 4 + 1)) + i * 3.9
    );

    return (
        <ElevationChart data={numbersArray}/>
    )
    // return (
    //     // <QueryClientProvider client={queryClient}>
    //     //     <MapboxWrapper currentPos={new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}>
    //     //         <SetupMapbox setOrigin={setOrigin} setDestination={setDestination} map={map} />
    //     //         <RenderBikeRoute origin={origin} destination={destination} map={renderRouteMap} />
    //     //     </MapboxWrapper>
    //     // </QueryClientProvider>
    // );
}
