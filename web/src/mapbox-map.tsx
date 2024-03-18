import { Fragment, useEffect, useRef, useState } from "react";
import mapboxgl, { type GeoJSONSource, type MapMouseEvent, type MapTouchEvent } from "mapbox-gl";
import { ColorLegend, TimeColorMapper } from "./colors";
import { mvtUrl } from "./dev-api";
import { getDetails, type DetailResponse } from "./get_data";
import { DetailPopup, type TripDetailsTransit } from "./format-details";
import track from "./analytics";
import { installDoubleTap } from "./double-tap-recognizer";
import { GIF_RENDER } from "@/gif-generator";

export const defaultColor = "rgba(143,143,143,0.13)";

export const EMPTY_GEOJSON: GeoJSON.FeatureCollection = {
    type: "FeatureCollection",
    features: [],
};

const TRANSIT_LAYER_ID = "transit-layer";
const GEOJSON_PATH_SOURCE_ID = "geojson-path";
const GEOJSON_PATH_LAYER_ID = "geojson-path-layer";
const GEOJSON_CIRCLE_LAYER_ID = "geojson-circle-layer";

function addMVTLayer(currentMap: mapboxgl.Map) {
    if (currentMap.getLayer(TRANSIT_LAYER_ID)) currentMap.removeLayer(TRANSIT_LAYER_ID);
    if (currentMap.getSource(TRANSIT_LAYER_ID)) currentMap.removeSource(TRANSIT_LAYER_ID);
    currentMap.addSource(TRANSIT_LAYER_ID, {
        type: "vector",
        // Use extension .bin to enable Cloudflare caching (doesn't cache on .pbf extension)
        tiles: [`${mvtUrl}/all_cities/{z}/{x}/{y}.bin`],
    });

    currentMap.addLayer({
        id: TRANSIT_LAYER_ID,
        type: "line",
        source: TRANSIT_LAYER_ID,
        "source-layer": "all_cities",
        layout: {
            "line-cap": "round",
            "line-join": "round",
        },
        paint: {
            "line-opacity": 0.47,
            "line-color": defaultColor,
            "line-width": 4.0,
        },
    });
}

function addGeoJsonLayer(currentMap: mapboxgl.Map): GeoJSONSource {
    currentMap.addSource(GEOJSON_PATH_SOURCE_ID, {
        type: "geojson",
    });

    currentMap.addLayer({
        id: GEOJSON_PATH_LAYER_ID,
        type: "line",
        source: GEOJSON_PATH_SOURCE_ID,
        layout: {
            "line-join": "round",
            "line-cap": "butt",
        },
        paint: {
            "line-color": ["get", "color"],
            "line-width": ["get", "line_width"],
            "line-opacity": 0.6,
        },
    });
    currentMap.addLayer({
        id: GEOJSON_CIRCLE_LAYER_ID,
        type: "circle",
        source: GEOJSON_PATH_SOURCE_ID,
        paint: {
            "circle-color": ["get", "color"],
            "circle-radius": 5.2,
        },
        filter: ["==", "$type", "Point"],
    });

    return currentMap.getSource(GEOJSON_PATH_SOURCE_ID) as GeoJSONSource;
}

function bufferPoint(point: mapboxgl.Point): [mapboxgl.Point, mapboxgl.Point] {
    const buffer = new mapboxgl.Point(3, 3);
    return [point.sub(buffer), point.add(buffer)];
}

function isTouchDevice() {
    return (
        // @ts-expect-error navigator
        "ontouchstart" in window || navigator.maxTouchPoints > 0 || navigator.msMaxTouchPoints > 0
    );
}

function setupMapboxMap(
    currentMap: mapboxgl.Map,
    setLatLng: (latlng: mapboxgl.LngLat) => void,
    getTimeData: () => TimeColorMapper,
    onMapLoaded: () => void,
    setDetailPopupInfo: (details: TripDetailsTransit[] | null, seconds: number | null) => void
) {
    currentMap.on("load", () => {
        addMVTLayer(currentMap);
        const geojsonSource = addGeoJsonLayer(currentMap);

        let abort = new AbortController();

        const removeHoverDetails = () => {
            abort.abort();
            currentMap.getCanvas().style.cursor = "";
            geojsonSource.setData(EMPTY_GEOJSON);
            setDetailPopupInfo(null, null);
        };

        document.addEventListener("keydown", (event) => {
            if (event.key === "Escape") {
                removeHoverDetails();
            }
        });

        const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent) || isTouchDevice();

        const handleDoubleClick = (e: MapMouseEvent | MapTouchEvent) => {
            e.preventDefault();
            track("dblclick-map-origin-change", {
                location: (e as MapMouseEvent).lngLat.toString(),
            });
            setLatLng((e as MapMouseEvent).lngLat);
        };
        if (isMobile) {
            installDoubleTap(currentMap, handleDoubleClick as (evt: MapTouchEvent) => void);
        } else {
            currentMap.on("dblclick", handleDoubleClick as (evt: MapMouseEvent) => void);
        }

        const handleHover = (e: MapMouseEvent) => {
            if (e.originalEvent.altKey || GIF_RENDER) {
                return;
            }

            abort.abort();
            abort = new AbortController();

            const nearbyFeatures = currentMap.queryRenderedFeatures(bufferPoint(e.point), {
                layers: [TRANSIT_LAYER_ID],
            });
            if (nearbyFeatures.length === 0) {
                if (e.type === "click") removeHoverDetails();
                return;
            }

            currentMap.getCanvas().style.cursor = "crosshair";
            const feature = nearbyFeatures[0];
            if (!feature.id) return;

            const seconds = getTimeData().raw[feature.id];

            if (!seconds) return;

            getDetails(getTimeData(), e.lngLat, abort.signal)
                .then((detailResponse: DetailResponse) => {
                    const details: TripDetailsTransit[] = detailResponse.details;
                    setDetailPopupInfo(details, seconds);

                    const path: GeoJSON.Feature = detailResponse.path;

                    track("hover-get-path", { location: e.lngLat.toString() });
                    if (path) {
                        geojsonSource.setData(path);
                    }
                })
                .catch((e) => {
                    if (e.toString().includes("SyntaxError: Unexpected token")) {
                        alert("Unexpected error. Please refresh the page and try again.");
                        // window.location.reload();
                    }
                    if (e.toString().includes("aborted a request")) {
                        return;
                    }
                    throw e;
                });
        };

        currentMap.on("mouseover", TRANSIT_LAYER_ID, handleHover);
        currentMap.on("click", handleHover);
        currentMap.on("mouseleave", TRANSIT_LAYER_ID, removeHoverDetails);

        onMapLoaded();
    });
}

export async function setAndColorNewOriginLocation(currentLatLng: mapboxgl.LngLat, currentOptions: any) {
    return await TimeColorMapper.fetch(
        currentLatLng,
        currentOptions.startTime,
        currentOptions.duration,
        currentOptions.agencies,
        currentOptions.modes,
        currentOptions.transferPenalty
        // currentOptions.minDuration
    );
}

interface MapboxMapProps {
    timeData: TimeColorMapper
    paintProperty: Record<string, string>
    setLatLng: (latlng: mapboxgl.LngLat) => void
    setSpinnerLoading: (loading: boolean) => void
    currentPos: mapboxgl.LngLat
}

export function MapboxMap({ timeData, paintProperty, setLatLng, setSpinnerLoading, currentPos }: MapboxMapProps) {
    const timeDataRef = useRef<TimeColorMapper | null>(null);
    const [map, setMap] = useState<mapboxgl.Map | null>(null);
    const [mapboxLoading, setMapboxLoading] = useState(true);
    const mapContainer = useRef<HTMLDivElement | null>(null);
    const [rerender, setRerender] = useState(false);

    const [detailPopup, setDetailPopup] = useState<{
        details: TripDetailsTransit[]
        seconds: number
    } | null>(null);

    timeDataRef.current = timeData;

    const getTimeData = (): TimeColorMapper => {
        if (timeDataRef.current != null) {
            return timeDataRef.current;
        } else {
            throw Error("TimeData is undefined right now");
        }
    };

    const setDetailPopupInfo = (details: TripDetailsTransit[] | null, seconds: number | null) => {
        if (!details || !seconds) setDetailPopup(null);
        else {
            setDetailPopup({
                details,
                seconds,
            });
        }
    };

    useEffect(() => {
        if (mapContainer.current == null || map !== null) return;

        mapboxgl.accessToken =
            "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";

        const mapInstance = new mapboxgl.Map({
            container: mapContainer.current,
            style: "mapbox://styles/mapbox/dark-v11",
            center: currentPos,
            zoom: 10.98,
            preserveDrawingBuffer: true,
        });
        setMap(mapInstance);
        mapInstance.doubleClickZoom.disable();
        setupMapboxMap(
            mapInstance,
            setLatLng,
            getTimeData,
            () => {
                setMapboxLoading(false);
            },
            setDetailPopupInfo
        );
    }, []);

    useEffect(() => {
        if (mapboxLoading || !paintProperty || !map) return;

        // timeData.current = paintProperty;

        let shouldRetry = false;
        const handleError = (err: mapboxgl.ErrorEvent) => {
            if (
                err.error.message.includes(
                    " does not exist in the map's style and cannot be styled."
                )
            ) {
                shouldRetry = true;
            }
            console.log("Error!! ", err);
        };
        map.once("error", handleError);

        map.setPaintProperty(TRANSIT_LAYER_ID, "line-color", [
            "coalesce",
            ["get", ["to-string", ["id"]], ["literal", paintProperty]],
            defaultColor,
        ]);

        const geojsonSource = map.getSource(GEOJSON_PATH_SOURCE_ID);
        if (geojsonSource && geojsonSource.type === "geojson") {
            geojsonSource.setData(EMPTY_GEOJSON);
        }

        if (shouldRetry) {
            console.log("Retrying...");
            addMVTLayer(map);
            new Promise((resolve) => setTimeout(resolve, 2000))
                .then(() => {
                    setRerender(!rerender);
                })
                .catch((e) => {
                    throw e;
                });
        }

        map.off("error", handleError);

        map.once("render", () => {
            // Takes some time for the map to update
            setTimeout(() => { setSpinnerLoading(false); }, 300);
        });
    }, [paintProperty, mapboxLoading, rerender]);

    useEffect(() => {
        if (map == null) return;
        map.setCenter(currentPos);
        map.setZoom(11);
    }, [currentPos]);

    return (
        <Fragment>
            {detailPopup != null && (
                <DetailPopup details={detailPopup.details} arrival_time={detailPopup.seconds} />
            )}

            {timeData && <ColorLegend tcm={timeData} currentHover={detailPopup?.seconds} />}

            <div ref={mapContainer} className="map w-screen h-screen overflow-none" />
        </Fragment>
    );
}
