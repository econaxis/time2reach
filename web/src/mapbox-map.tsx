import { useEffect, useRef, useState } from "preact/hooks"
import mapboxgl, { type GeoJSONSource } from "mapbox-gl"
import { ColorLegend, TimeColorMapper } from "./colors"
import { mvtUrl } from "./dev-api"
import { getDetails } from "./get_data"
import { DetailPopup, type TripDetailsTransit } from "./format-details"
import { startingLocation } from "./app"
import { Fragment } from "preact"
import track from "./analytics"

export const defaultColor = "rgba(182,182,182,0.14)"

const EMPTY_GEOJSON: GeoJSON.FeatureCollection = {
    type: "FeatureCollection",
    features: []
}

function addMVTLayer (currentMap: mapboxgl.Map) {
    currentMap.addSource("some id", {
        type: "vector",
        tiles: [`${mvtUrl}/all_cities/{z}/{x}/{y}.pbf`]
    })

    currentMap.addLayer({
        id: "transit-layer", // Layer ID
        type: "line",
        source: "some id", // ID of the tile source created above
        "source-layer": "all_cities",
        layout: {
            "line-cap": "round",
            "line-join": "round"
        },
        paint: {
            "line-opacity": 0.4,
            // "line-color": defaultColor,
            "line-color": "#8f8f8f",
            "line-width": 3.5
        }
    })
}

function addGeoJsonLayer (currentMap: mapboxgl.Map): GeoJSONSource {
    currentMap.addSource("geojson-path", {
        type: "geojson"
    })

    currentMap.addLayer({
        id: "geojson-path-layer",
        type: "line",
        source: "geojson-path",
        layout: {
            "line-join": "round",
            "line-cap": "butt"
        },
        paint: {
            "line-color": ["get", "color"],
            "line-width": ["get", "line_width"],
            "line-opacity": 0.7
        }
    })

    currentMap.addLayer({
        id: "geojson-circle-layer",
        type: "circle",
        source: "geojson-path",
        paint: {
            "circle-color": ["get", "color"],
            "circle-radius": 5.2
        },
        filter: ["==", "$type", "Point"]
    })

    return currentMap.getSource("geojson-path") as GeoJSONSource
}

function bufferPoint (point: mapboxgl.Point): [mapboxgl.Point, mapboxgl.Point] {
    const buffer = new mapboxgl.Point(5, 5)
    return [point.sub(buffer), point.add(buffer)]
}

function setupMapboxMap (currentMap: mapboxgl.Map, setLatLng: (latlng: mapboxgl.LngLat) => void, getTimeData: () => TimeColorMapper, doneCallback: () => void, setDetailPopupInfo: (TripDetailsTransit?, number?) => void) {
    currentMap.on("load", () => {
        addMVTLayer(currentMap)

        const geojsonSource = addGeoJsonLayer(currentMap)

        currentMap.on("dblclick", (e) => {
            e.preventDefault()
            track("dblclick-map-origin-change", { location: e.lngLat.toString() })
            setLatLng(e.lngLat)
        })

        const hoverCallback = (e) => {
            const nearbyFeatures = currentMap.queryRenderedFeatures(bufferPoint(e.point))
            if (nearbyFeatures.length === 0) {
                console.log("no nearby features found")
                return
            }

            currentMap.getCanvas().style.cursor = "crosshair"
            const feature = nearbyFeatures[0]
            if (!feature.id) return

            const seconds = getTimeData().raw[feature.id]

            if (!seconds) return

            getDetails(
                getTimeData(),
                e.lngLat
            ).then(detailResponse => {
                const details: TripDetailsTransit[] = detailResponse.details
                setDetailPopupInfo(details, seconds)

                const path: GeoJSON.Feature = detailResponse.path

                track('hover-get-path', { location: e.lngLat.toString() })
                if (path) {
                    console.log("Setting geojson data", path)
                    geojsonSource.setData(path)
                }
            }).catch(e => {
                throw e
            })
        }
        currentMap.on("mouseover", "transit-layer", hoverCallback)
        currentMap.on("click", "transit-layer", hoverCallback)
        currentMap.on("mouseleave", "transit-layer", () => {
            currentMap.getCanvas().style.cursor = ""
            geojsonSource.setData(EMPTY_GEOJSON)
            setDetailPopupInfo(null, null)
        })

        doneCallback()
    })
}

export function MapboxMap ({
                              currentOptions,
                              currentLatLng,
                              setLatLng,
                              currentPos,
                              setSpinnerLoading
                          }) {
    const [map, setMap] = useState<mapboxgl.Map | null>(null)
    const [mapboxLoading, setMapboxLoading] = useState(true)
    const timeData = useRef<TimeColorMapper | null>(null)
    const [timeDataState, setTimeDataState] = useState<any>(null)
    const mapContainer = useRef<HTMLElement | null>(null)

    const [detailPopup, setDetailPopup] = useState<{ details: TripDetailsTransit[], seconds: number } | null>(null)

    const getTimeData = (): TimeColorMapper => {
        if (timeData.current != null) {
            return timeData.current
        } else {
            throw Error("TimeData is undefined right now")
        }
    }

    const setDetailPopupInfo = (details: TripDetailsTransit[], seconds) => {
        if (!details || !seconds) setDetailPopup(null)
        else {
            setDetailPopup({
                details, seconds
            })
        }
    }

    useEffect(() => {
        // Init mapbox gl map here.
        if (mapContainer.current == null) return

        mapboxgl.accessToken =
            "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A"

        const map = new mapboxgl.Map({
            container: mapContainer.current, // container ID
            style: "mapbox://styles/mapbox/dark-v11", // style URL
            center: startingLocation, // starting position [lng, lat]
            zoom: 12 // starting zoom
        })
        setMap(map)
        setupMapboxMap(map, setLatLng, getTimeData, () => {
            setMapboxLoading(false)
        }, setDetailPopupInfo)
    }, [])

    useEffect(() => {
        if (!currentOptions?.agencies) {
            console.log("CO agencies not defined", currentOptions)
            return
        }
        if (!currentLatLng) return
        if (mapboxLoading) return
        if (!map) return

        setSpinnerLoading(true)
        TimeColorMapper.fetch(currentLatLng, currentOptions.startTime, currentOptions.duration, currentOptions.agencies, currentOptions.modes).then(data => {
            timeData.current = data
            setTimeDataState(timeData.current)

            console.log("Setting paint property")
            map.setPaintProperty("transit-layer", "line-color", [
                "coalesce",
                ["get", ["to-string", ["id"]], ["literal", data.m]],
                defaultColor
            ])

            map.once("render", () => {
                // Takes roughly 300 ms for the map to update
                setTimeout(() => setSpinnerLoading(false), 300)
            })
        }).catch(err => {
            console.error("Error in timecolormapper.fetch")
            throw err
        })
    }, [currentOptions, currentLatLng, map, mapboxLoading])

    useEffect(() => {
        if (!map) return
        map.setCenter(currentPos)
        map.setZoom(11)
    }, [currentPos])

    return <Fragment>
        {detailPopup ? <DetailPopup details={detailPopup.details} arrival_time={detailPopup.seconds} /> : null}

        {timeDataState ? <ColorLegend tcm={timeDataState} currentHover={detailPopup?.seconds} /> : null}

        {/* @ts-expect-error ref and mapContainer doesn't match types */}
        <div ref={mapContainer} className="map w-screen h-screen overflow-none" />
    </Fragment>
}
