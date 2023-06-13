import { useEffect, useRef, useState } from 'preact/hooks'
import mapboxgl from 'mapbox-gl'
import { DetailPopup, type TripDetailsTransit } from './format-details'
import { getDetails } from './get_data'
import { defaultColor } from './ol'
import { render } from 'preact'
import { TimeColorMapper } from './colors'

import './style.css'
import { CityPillContainer } from './city-pill'
import { QueryClient, QueryClientProvider, useQuery } from 'react-query'
import { TimeSlider } from './time-slider'
import { baseUrl, mvtUrl } from "./dev-api";

interface Agency {
    agencyCode: string
    agencyLongName: string
    city: string
}

export function AgencyEntry ({
    agencyCode,
    agencyLongName,
    setSelectValue
}: Agency | object) {
    // agencyCode: TTC/YRT/UP ...
    // agencyLongName: Toronto Transit Commission

    const onChange = (element: any) => {
        setSelectValue(agencyCode, element.target.checked)
    }

    const id = `agency-${agencyCode}`
    return (
        <div>
            <input
                id={id}
                type="checkbox"
                className="checkbox"
                onChange={onChange}
                defaultChecked
            />
            <label htmlFor={id} className="ml-1 text-gray-900">
                {agencyLongName}
            </label>
        </div>
    )
}

export function Header ({ children }) {
    return (
        <h2 className="font-medium text-lg font-bold border-b mt-3">
            {children}
        </h2>
    )
}

export function AgencyForm ({
    agencies,
    header,
    updateValues
}) {
    const values = useRef(Object.fromEntries(agencies.map(ag => [ag.agencyCode, true])))

    useEffect(() => {
        updateValues(values.current)
    }, [])
    const setSelectValue = (value, status) => {
        values.current[value] = status
        updateValues(values.current)
    }
    const agencyList = agencies.filter(ag => ag.shouldShow || ag.shouldShow === undefined).map((ag) => (
        <AgencyEntry {...ag} setSelectValue={setSelectValue} />
    ))

    return (
        <div>
            <Header>{header}</Header>

            <form id="agency-form" className="mt-2">
                {agencyList}
            </form>
        </div>
    )
}

export function Sidebar ({ children, zi }) {
    return (
        <div className="absolute top-0 right-0 m-5 w-3/12 p-5 bg-white border border-gray-200 rounded-lg shadow" style={{ zIndex: zi || 0 }}>
            <p className="text-gray-700">
                Double click anywhere to see how far you can go by public
                transit.
            </p>
            {children}
        </div>
    )
}

async function fetchAgencies (): Promise<Agency[]> {
    console.log('fetching agencies')
    const result = await fetch(`${baseUrl}/agencies`)
    const json = await result.json()
    console.log('json is', json)
    return json.map(agency => {
        return {
            agencyCode: agency.short_code,
            agencyLongName: agency.public_name,
            city: agency.city
        }
    })
}

function useAgencies () {
    return useQuery('agencies', fetchAgencies)
}

const MODES = [
    {
        agencyCode: 'bus',
        agencyLongName: 'Bus'
    },
    {
        agencyCode: 'subway',
        agencyLongName: 'Subway'
    },
    {
        agencyCode: 'tram',
        agencyLongName: 'Tram'
    },
    {
        agencyCode: 'rail',
        agencyLongName: 'Train'
    }
]

function setupMapboxMap (currentMap: mapboxgl.Map, setLatLng: (latlng: mapboxgl.LngLat) => void, getTimeData: () => TimeColorMapper) {
    currentMap.on('load', async () => {
        currentMap.addSource('some id', {
            type: 'vector',
            // tiles: ['http://127.0.0.1:6767/all_cities/{z}/{x}/{y}.pbf']
            tiles: [`${mvtUrl}/all_cities/{z}/{x}/{y}.pbf`]
        })

        currentMap.addLayer({
            id: 'transit-layer', // Layer ID
            type: 'line',
            source: 'some id', // ID of the tile source created above
            'source-layer': 'all_cities',
            layout: {
                'line-cap': 'round',
                'line-join': 'round'
            },
            paint: {
                'line-opacity': 0.3,
                'line-color': defaultColor,
                'line-width': 3.3
            }
        })

        currentMap.on('dblclick', async (e) => {
            e.preventDefault()
            setLatLng(e.lngLat)
        })

        const popup = new mapboxgl.Popup({
            maxWidth: 'none'
        })

        let currentTask
        currentMap.on('mouseover', 'transit-layer', async (e) => {
            const nearbyFeatures = currentMap.queryRenderedFeatures(e.point)
            if (nearbyFeatures.length === 0) return

            if (currentTask) clearTimeout(currentTask)

            currentMap.getCanvas().style.cursor = 'crosshair'
            currentTask = setTimeout(async () => {
                const feature = nearbyFeatures[0]
                const seconds = getTimeData().raw[feature.id]

                if (!seconds) return

                const details: TripDetailsTransit[] = await getDetails(
                    getTimeData(),
                    e.lngLat
                )

                const node = document.createElement('div')
                const detailPopup = <DetailPopup details={details} arrival_time={seconds}></DetailPopup>
                render(detailPopup, node)
                popup.setDOMContent(node)
                popup.setLngLat(e.lngLat)
                popup.addTo(currentMap)
            }, 300)
        })
        currentMap.on('mouseleave', 'transit-layer', (e) => {
            currentMap.getCanvas().style.cursor = ''
            clearTimeout(currentTask)
            popup.remove()
            currentTask = undefined
        })
    })
}

export function MapboxMap ({
    currentOptions,
    currentLatLng,
    setLatLng,
    currentPos
}) {
    const [map, setMap] = useState<mapboxgl.Map | null>(null)
    const [loading, setLoading] = useState(true)
    const timeData = useRef<TimeColorMapper | null>(null)
    const mapContainer = useRef<HTMLElement | null>(null)

    const getTimeData = () => {
        if (timeData.current != null) {
            return timeData.current
        } else {
            throw Error('TimeData is undefined right now')
        }
    }

    useEffect(() => {
        // Init mapbox gl map here.
        if (mapContainer.current == null) return

        mapboxgl.accessToken =
            'pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A'

        const map = new mapboxgl.Map({
            container: mapContainer.current, // container ID
            style: 'mapbox://styles/mapbox/dark-v11', // style URL
            center: startingLocation, // starting position [lng, lat]
            zoom: 12 // starting zoom
        })
        setMap(map)

        const currentMap = map

        setupMapboxMap(currentMap, setLatLng, getTimeData)

        currentMap.on('load', () => {
            setLoading(false)
        })
    }, [])

    useEffect(() => {
        if (!currentOptions) return
        if (!currentLatLng) return
        if (loading) return
        if (!map) return

        void TimeColorMapper.fetch(currentLatLng, currentOptions.startTime, currentOptions.duration, currentOptions.agencies, currentOptions.modes).then(data => {
            timeData.current = data

            map.setPaintProperty('transit-layer', 'line-color', [
                'coalesce',
                ['get', ['to-string', ['id']], ['literal', data.m]],
                defaultColor
            ])
        })
    }, [currentOptions, currentLatLng, map, loading])

    useEffect(() => {
        if (!map) return
        map.setCenter(currentPos)
        map.setZoom(11)
    }, [currentPos])

    return <div ref={mapContainer} className="map w-screen h-screen overflow-none"></div>
}

export function ControlSidebar ({ setOptions, currentCity }) {
    const agencies = useRef<object | null>(null)
    const modes = useRef<object | null>(null)

    const [duration, setDuration] = useState(3600)
    const [startTime, setStartTime] = useState(17 * 3600)


    useEffect(() => {
        triggerRefetch()
    }, [duration, startTime])
    const onAgencyChange = (agencies1: object) => {
        console.log('onAgencyChange')
        agencies.current = agencies1
        triggerRefetch()
    }

    const onModeChange = (modes1: object) => {
        console.log('onModeChange')
        modes.current = modes1
        triggerRefetch()
    }

    const triggerRefetch = () => {
        setOptions({
            duration,
            startTime,
            agencies: agencies.current,
            modes: modes.current
        })
    }

    const {
        isLoading,
        data
    } = useAgencies()

    if (isLoading) return null
    if (!data) throw new Error("data is null")

    const filtered = data.map(ag => {
        return {
            shouldShow: ag.city === currentCity,
            ...ag
        }
    })

    console.log("Agencies", filtered)

    return <Sidebar zi={10}>
        <AgencyForm
            agencies={filtered}
            header="Agencies"
            updateValues={onAgencyChange}
        />

        <AgencyForm agencies={MODES} header="Modes"
                    updateValues={onModeChange}
        />

        <TimeSlider duration={duration} setDuration={setDuration} startTime={startTime} setStartTime={setStartTime} />
    </Sidebar>
}

const CITY_LOCATION = {
    Toronto: new mapboxgl.LngLat(-79.3832, 43.6532),
    "New York City": new mapboxgl.LngLat(-74.0060, 40.7128),
    Montreal: new mapboxgl.LngLat(-73.5674, 45.5019),
    Vancouver: new mapboxgl.LngLat(-123.1207, 49.2827)
}

export const startingLocation = CITY_LOCATION.Toronto

export function App () {
    const queryClient = new QueryClient({})

    const [currentOptions, setCurrentOptions] = useState(null)
    const [currentStartingLoc, setCurrentStartingLoc] = useState(startingLocation)
    const [currentCity, setCurrentCity] = useState("Toronto")

    const cityLocation = CITY_LOCATION[currentCity]
    const setCityFromPill = (cityName: string) => {
        setCurrentCity(cityName)
        setCurrentStartingLoc(CITY_LOCATION[cityName])
    }

    return (
        <QueryClientProvider client={queryClient}>
            <CityPillContainer cities={['Toronto', 'Montreal', 'Vancouver', 'New York City']}
                               setLocation={setCityFromPill} currentCity={currentCity} />
            <MapboxMap currentOptions={currentOptions} currentLatLng={currentStartingLoc}
                       setLatLng={setCurrentStartingLoc}
                       currentPos={cityLocation} />
            <ControlSidebar setOptions={setCurrentOptions} currentCity={currentCity} />
        </QueryClientProvider>
    )
}
