import { useEffect, useRef, useState } from 'preact/hooks'
import mapboxgl from 'mapbox-gl'
import { DetailPopup, type TripDetailsTransit } from './format-details'
import { getDetails } from './get_data'
import { defaultColor, startingLocation } from './ol'
import { render } from 'preact'
import { TimeColorMapper } from './colors'

import './style.css'
import { CityPillContainer } from './city-pill'
import { QueryClient, QueryClientProvider, useQuery } from 'react-query'

interface Agency {
    agencyCode: string
    agencyLongName: string
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
    const agencyList = agencies.map((ag) => (
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

export function Sidebar ({ children }) {
    return (
        <div className="absolute top-0 right-0 m-5 max-w-sm p-5 bg-white border border-gray-200 rounded-lg shadow">
            <p className="text-gray-700">
                Double click anywhere to see how far you can go by public
                transit.
            </p>
            {children}
        </div>
    )
}

async function fetchAgencies () {
    console.log('fetching agencies')
    const result = await fetch('http://localhost:3030/agencies')
    const json = await result.json()
    console.log('json is', json)
    return json.map(agency => {
        return {
            agencyCode: agency.short_code,
            agencyLongName: agency.public_name
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
            tiles: ['http://127.0.0.1:6767/all_cities/{z}/{x}/{y}.pbf']
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
        currentMap.on('mouseleave', 'transit-layer', () => {
            currentMap.getCanvas().style.cursor = ''
            clearTimeout(currentTask)
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

        console.log('Fetching new data', currentLatLng, currentOptions)
        void TimeColorMapper.fetch(currentLatLng, currentOptions.duration, currentOptions.agencies, currentOptions.modes).then(data => {
            timeData.current = data;
            (map as mapboxgl.Map).setPaintProperty('transit-layer', 'line-color', [
                'coalesce',
                ['get', ['to-string', ['id']], ['literal', data.m]],
                defaultColor
            ])
        })
    }, [currentOptions, currentLatLng, map, loading])

    useEffect(() => {
        if (!map) return

        console.log('Setting center', currentPos)
        map.setCenter(currentPos)
        map.setZoom(11)
    }, [currentPos])

    return <div ref={mapContainer} className="map w-screen h-screen overflow-none"></div>
}

// eslint-disable-next-line react/prop-types
export function TimeSlider ({ setDuration }) {
    const defaultDurationRange = 3600
    const onChange = (element) => {
        setDuration(parseInt(element.target.value))
    }
    useEffect(() => {
        console.log('Setting duration')
        setDuration(3600)
    }, [])
    return (
        <div className="mt-2">
            <Header>Time Settings</Header>

            <div className="mt-2">
                <div>
                    <label
                        htmlFor="duration-range"
                        className="float-left mb-1 text-sm font-medium text-gray-900"
                    >
                        Maximum duration of trip
                    </label>
                    <span
                        id="duration-label"
                        className="float-right inline-block mb-1 text-sm font-light text-gray-700"
                    >1:00</span>
                </div>
                <input
                    id="duration-range"
                    type="range"
                    min="1800"
                    max="5400"
                    value={defaultDurationRange.toString()}
                    className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    onChange={onChange}
                />
            </div>
        </div>
    )
}

export function ControlSidebar ({ setOptions }) {
    const agencies = useRef<object | null>(null)
    const modes = useRef<object | null>(null)
    const duration = useRef<number | null>(null)

    const onDurationChange = (durationSecs: number) => {
        console.log('onDurationChange')
        duration.current = durationSecs
        triggerRefetch()
    }

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
            duration: duration.current,
            agencies: agencies.current,
            modes: modes.current
        })
    }

    const {
        isLoading,
        data,
        error
    } = useAgencies()

    if (isLoading) return

    console.log('got agencies', data, error)

    return <Sidebar>
        <AgencyForm
            agencies={data}
            header="Agencies"
            updateValues={onAgencyChange}
        />

        <AgencyForm agencies={MODES} header="Modes"
                    updateValues={onModeChange}
        />

        <TimeSlider setDuration={onDurationChange} />
    </Sidebar>
}

export function App () {
    const queryClient = new QueryClient({})

    const [currentOptions, setCurrentOptions] = useState(null)
    const [currentStartingLoc, setCurrentStartingLoc] = useState(startingLocation)
    const [currentPos, setCurrentPos] = useState(startingLocation)

    return (
        <QueryClientProvider client={queryClient}>
            <CityPillContainer cities={['Toronto', 'Montreal', 'Vancouver', 'New York City']}
                               setLocation={setCurrentPos} />
            <MapboxMap currentOptions={currentOptions} currentLatLng={currentStartingLoc}
                       setLatLng={setCurrentStartingLoc}
                       currentPos={currentPos} />
            <ControlSidebar setOptions={setCurrentOptions} />
        </QueryClientProvider>
    )
}
