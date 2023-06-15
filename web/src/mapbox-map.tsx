import { useEffect, useRef, useState } from "preact/hooks"
import mapboxgl from "mapbox-gl"
import { TimeColorMapper } from "./colors"
import { defaultColor } from "./ol"
import { mvtUrl } from "./dev-api"
import { getDetails } from "./get_data"
import { DetailPopup, type TripDetailsTransit } from "./format-details";
import { startingLocation } from "./app"
import { Fragment, render } from "preact";

function setupMapboxMap (currentMap: mapboxgl.Map, setLatLng: (latlng: mapboxgl.LngLat) => void, getTimeData: () => TimeColorMapper, doneCallback: () => void, setDetailPopupInfo: (TripDetailsTransit, number) => void) {
    currentMap.on("load", async () => {
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
                "line-opacity": 0.3,
                "line-color": defaultColor,
                "line-width": 3.3
            }
        })

        currentMap.addSource("geojson-path", {
            type: "geojson"
        })

        const geojsonSource = currentMap.getSource("geojson-path")

        currentMap.addLayer({
            id: "geojson-path-layer",
            type: "line",
            source: "geojson-path",
            layout: {
                "line-join": "round",
                "line-cap": "round"
            },
            paint: {
                "line-color": "#888",
                "line-width": 4
            }
        })

        currentMap.on("dblclick", async (e) => {
            e.preventDefault()
            setLatLng(e.lngLat)
        })


        let currentTask
        currentMap.on("mouseover", "transit-layer", async (e) => {
            const nearbyFeatures = currentMap.queryRenderedFeatures(e.point)
            if (nearbyFeatures.length === 0) return

            if (currentTask) clearTimeout(currentTask)

            currentMap.getCanvas().style.cursor = "crosshair"
            currentTask = setTimeout(async () => {
                const feature = nearbyFeatures[0]
                const seconds = getTimeData().raw[feature.id]

                if (!seconds) return

                const detailResponse = await getDetails(
                    getTimeData(),
                    e.lngLat
                )

                const details: TripDetailsTransit[] = detailResponse.details

                setDetailPopupInfo(details, seconds)

                const path: object = detailResponse.path

                geojsonSource.setData(path)

                // render(detailPopup, node)
                // popup.setDOMContent(node)
                // // popup.setLngLat(e.lngLat)
                // popup.setOffset({
                //     center: [10, 10]
                // })
                // popup.addTo(currentMap)
            }, 50)
        })
        currentMap.on("mouseleave", "transit-layer", (e) => {
            currentMap.getCanvas().style.cursor = ""
            clearTimeout(currentTask)
            currentTask = undefined
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
    const mapContainer = useRef<HTMLElement | null>(null)

    const [detailPopup, setDetailPopup] = useState<any>(null)

    const getTimeData = (): TimeColorMapper => {
        if (timeData.current != null) {
            return timeData.current
        } else {
            throw Error("TimeData is undefined right now")
        }
    }

    const setDetailPopupInfo = (details, seconds) => {
        // const detailPopup = <DetailPopup details={details} arrival_time={seconds}></DetailPopup>
        setDetailPopup({
            details, seconds
        })
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

        const currentMap = map

        setupMapboxMap(currentMap, setLatLng, getTimeData, () => {
            console.log("Done render")
            setMapboxLoading(false)
        }, setDetailPopupInfo)
    }, [])

    useEffect(() => {
        if (!currentOptions) return
        if (!currentLatLng) return
        if (mapboxLoading) return
        if (!map) return

        setSpinnerLoading(true)
        void TimeColorMapper.fetch(currentLatLng, currentOptions.startTime, currentOptions.duration, currentOptions.agencies, currentOptions.modes).then(data => {
            timeData.current = data

            map.setPaintProperty("transit-layer", "line-color", [
                "coalesce",
                ["get", ["to-string", ["id"]], ["literal", data.m]],
                defaultColor
            ])

            map.once("render", () => {
                setSpinnerLoading(false)
            })
        })
    }, [currentOptions, currentLatLng, map, mapboxLoading])

    useEffect(() => {
        if (!map) return
        map.setCenter(currentPos)
        map.setZoom(11)
    }, [currentPos])

    return <Fragment>
        {detailPopup ? <DetailPopup details={detailPopup.details} arrival_time={detailPopup.seconds}/> : null }
        <div ref={mapContainer} className="map w-screen h-screen overflow-none"></div>
    </Fragment>
}
