import { Fragment, useEffect, useRef, useState } from "react";
import mapboxgl, { type GeoJSONSource } from "mapbox-gl";
import { ColorLegend, TimeColorMapper } from "./colors";
import { mvtUrl } from "./dev-api";
import { getDetails } from "./get_data";
import { DetailPopup, type TripDetailsTransit } from "./format-details";
import track from "./analytics";
import { installDoubleTap } from "./double-tap-recognizer";

export const defaultColor = "rgba(143,143,143,0.13)";

const EMPTY_GEOJSON: GeoJSON.FeatureCollection = {
    type: "FeatureCollection",
    features: [],
};

function addMVTLayer(currentMap: mapboxgl.Map) {
    if (currentMap.getLayer("transit-layer")) currentMap.removeLayer("transit-layer");
    if (currentMap.getSource("some id")) currentMap.removeSource("some id");
    currentMap.addSource("some id", {
        type: "vector",
        // Use extension .bin to enable Cloudflare caching (doesn't cache on .pbf extension)
        tiles: [`${mvtUrl}/all_cities/{z}/{x}/{y}.bin`],
    });

    currentMap.addLayer({
        id: "transit-layer", // Layer ID
        type: "line",
        source: "some id", // ID of the tile source created above
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

const GEOJSON_PATH_SOURCEID = "geojson-path";

function addGeoJsonLayer(currentMap: mapboxgl.Map): GeoJSONSource {
    currentMap.addSource(GEOJSON_PATH_SOURCEID, {
        type: "geojson",
    });

    currentMap.addLayer({
        id: "geojson-path-layer",
        type: "line",
        source: GEOJSON_PATH_SOURCEID,
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
        id: "geojson-circle-layer",
        type: "circle",
        source: GEOJSON_PATH_SOURCEID,
        paint: {
            "circle-color": ["get", "color"],
            "circle-radius": 5.2,
        },
        filter: ["==", "$type", "Point"],
    });

    return currentMap.getSource(GEOJSON_PATH_SOURCEID) as GeoJSONSource;
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
    doneCallback: () => void,
    setDetailPopupInfo: (TripDetailsTransit?, number?) => void
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

        const dblClickHandler = (e) => {
            e.preventDefault();
            track("dblclick-map-origin-change", {
                location: e.lngLat.toString(),
            });
            setLatLng(e.lngLat);
        };
        if (isMobile) {
            installDoubleTap(currentMap, dblClickHandler);
        } else {
            currentMap.on("dblclick", dblClickHandler);
        }

        const hoverCallback = (e) => {
            if (e.originalEvent.altKey || GIF_RENDER) {
                return;
            }

            abort.abort();
            abort = new AbortController();

            const nearbyFeatures = currentMap.queryRenderedFeatures(bufferPoint(e.point), {
                layers: ["transit-layer"],
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
                .then((detailResponse) => {
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
                        alert("Unexpected error. Please refresh the page and try again.")
                        // window.location.reload();
                    }
                    if (e.toString().includes("aborted a request")) {
                        return;
                    }
                    throw e;
                });
        };

        currentMap.on("mouseover", "transit-layer", hoverCallback);
        currentMap.on("click", hoverCallback);
        currentMap.on("mouseleave", "transit-layer", removeHoverDetails);

        doneCallback();
    });
}

export async function setAndColorNewOriginLocation(currentLatLng, currentOptions) {
    return await TimeColorMapper.fetch(
        currentLatLng,
        currentOptions.startTime,
        currentOptions.duration,
        currentOptions.agencies,
        currentOptions.modes,
        currentOptions.minDuration
    );
}

export function MapboxMap({ timeData, paintProperty, setLatLng, setSpinnerLoading, currentPos }) {
    const timeDataRef = useRef<any>(null);
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

    const setDetailPopupInfo = (details: TripDetailsTransit[], seconds) => {
        if (!details || !seconds) setDetailPopup(null);
        else {
            setDetailPopup({
                details,
                seconds,
            });
        }
    };

    useEffect(() => {
        // Init mapbox gl map here.
        if (mapContainer.current == null) return;
        if (map !== null) return;

        mapboxgl.accessToken =
            "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";


        const map1 = new mapboxgl.Map({
            container: mapContainer.current, // container ID
            style: "mapbox://styles/mapbox/dark-v11", // style URL
            center: currentPos, // starting position [lng, lat]
            zoom: 10.98, // starting zoom
            preserveDrawingBuffer: true
        });
        setMap(map1);
        map1.doubleClickZoom.disable();
        setupMapboxMap(
            map1,
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

        timeData.current = paintProperty;

        let shouldRetry = false;
        const errHandler = (err) => {
            if (
                err.error.message.includes(
                    " does not exist in the map's style and cannot be styled."
                )
            ) {
                shouldRetry = true;
            }
            console.log("Error!! ", err);
        };
        map.once("error", errHandler);

        map.setPaintProperty("transit-layer", "line-color", [
            "coalesce",
            ["get", ["to-string", ["id"]], ["literal", paintProperty]],
            defaultColor,
        ]);

        const geojsonSource = map.getSource(GEOJSON_PATH_SOURCEID);
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

        map.off("error", errHandler);

        map.once("render", () => {
            // Takes some time for the map to update
            setTimeout(() => setSpinnerLoading(false), 300);
        });
    }, [paintProperty, mapboxLoading, rerender]);

    // useEffect(() => {
    //     if (map == null) return;
    //     map.setCenter(currentPos);
    //
    //     map.setZoom(11);
    // }, [currentPos]);

    // console.log("Center loc is ", map?.getCenter())
    return (
        <Fragment>
            {detailPopup != null ? (
                <DetailPopup details={detailPopup.details} arrival_time={detailPopup.seconds} />
            ) : null}

            {timeData ? <ColorLegend tcm={timeData} currentHover={detailPopup?.seconds} /> : null}

            <div ref={mapContainer} className="map w-screen h-screen overflow-none" />
        </Fragment>
    );
}
