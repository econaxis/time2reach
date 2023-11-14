import { Fragment, useEffect, useState } from "react";
import mapboxgl from "mapbox-gl";
import { MapboxWrapper } from "./mapbox-wrapper";
import { QueryClient, QueryClientProvider, useQuery } from "react-query";

export interface SetupProps {
    map: mapboxgl.Map | null
    setOrigin: (latLng: mapboxgl.LngLat) => void
    setDestination: (latLng: mapboxgl.LngLat) => void
}

export function SetupMapbox(props: SetupProps) {
    const { map, setOrigin, setDestination } = props;
    useEffect(() => {
        if (!map) {
            return;
        }
        console.log("SETTING UP MAPBOX");
        let isOrg = true;
        const dblClickHandler = (e) => {
            e.preventDefault();
            console.log("Double clicked!", isOrg);

            if (isOrg) {
                setOrigin(e.lngLat);
            } else {
                setDestination(e.lngLat);
            }
            isOrg = !isOrg;
        };
        map.on("dblclick", dblClickHandler);

        return () => {
            map.off("dblclick", dblClickHandler);
        };
    }, [map]);

    return <Fragment />;
}

export interface RenderStraightRouteProps {
    map: mapboxgl.Map | null
    origin: mapboxgl.LngLat | null
    destination: mapboxgl.LngLat | null
}

export interface RenderRouteProps {
    map: mapboxgl.Map | null
    routeData?: GeoJSON.FeatureCollection
}

export function RenderRoute(props: RenderRouteProps) {
    const { map, routeData } = props;

    useEffect(() => {
        if (!map) {
            return;
        }

        console.log("LOADING map")

        if (!map.getSource("route")) {
            map.addSource("route", {
                type: "geojson",
                data: "",
            });

            map.addLayer({
                id: "route",
                type: "line",
                source: "route",
                layout: {
                    "line-join": "round",
                    "line-cap": "round",
                },
                paint: {
                    "line-color": "#888",
                    "line-width": 8,
                },
            });
        }
    }, [map]);
    useEffect(() => {
        if (!map || !routeData) {
            return;
        }

        (map.getSource("route") as mapboxgl.GeoJSONSource).setData(routeData);
    }, [map, routeData]);

    return <Fragment />;
}

export function RenderStraightRoute(props: RenderStraightRouteProps) {
    const { origin, destination } = props;

    if (!origin || !destination) {
        return <Fragment />;
    }

    const routeData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: [
            {
                type: "Feature",
                properties: {},
                geometry: {
                    type: "LineString",
                    coordinates: [
                        [origin.lng, origin.lat],
                        [destination.lng, destination.lat],
                    ],
                },
            },
        ],
    };
    return <RenderRoute map={props.map} routeData={routeData}/>;
}

async function fetchBikeRoute(origin: mapboxgl.LngLat, destination: mapboxgl.LngLat) {
    const url = `http://localhost:3030/bike`
    const postData = {
        start: {
            latitude: origin.lat,
            longitude: origin.lng,
        },
        end: {
            latitude: destination.lat,
            longitude: destination.lng,
        },
    }
    const req = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(postData),
    })

    if (!req.ok) {
        throw new Error(`Failed to fetch bike route: ${req.status}  ${await req.text()}`);
    }
    return await req.json()
}

export interface RenderBikeRouteProps extends RenderStraightRouteProps {}
export function RenderBikeRoute(props: RenderBikeRouteProps) {
    const { origin, destination } = props;

    if (!origin || !destination) {
        return <Fragment />;
    }

    // Use react-query to query the bike route
    const { data, isLoading } = useQuery(["bike-route", origin, destination], async() => {
        return await fetchBikeRoute(origin, destination);
    });

    if (isLoading) {
        return <Fragment />;
    }

    const routeData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: [
            data
        ],
    };
    return <RenderRoute map={props.map} routeData={routeData}/>;
}

export function BikeMap() {
    const queryClient = new QueryClient({});

    const [origin, setOrigin] = useState<mapboxgl.LngLat | null>(null);
    const [destination, setDestination] = useState<mapboxgl.LngLat | null>(null);
    const [map, setMap] = useState<mapboxgl.Map | null>(null);

    const mapOnLoad = (map: mapboxgl.Map) => {
        setMap(map);
    };

    let renderRouteMap: mapboxgl.Map | null = null;
    if (map != null) {
        renderRouteMap = map;
    }

    return (
        <QueryClientProvider client={queryClient}>
        <MapboxWrapper currentPos={ new mapboxgl.LngLat(-122.4194, 37.7749)} onLoad={mapOnLoad}>
            <SetupMapbox setOrigin={setOrigin} setDestination={setDestination} map={map} />
            <RenderBikeRoute origin={origin} destination={destination} map={renderRouteMap} />
        </MapboxWrapper>
        </QueryClientProvider>
    );
}
