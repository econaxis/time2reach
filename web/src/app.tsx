import { useState } from "preact/hooks"
import mapboxgl from "mapbox-gl"

import "./style.css"
import { CityPillContainer } from "./city-pill"
import { QueryClient, QueryClientProvider } from "react-query"
import { LoadingSpinner } from "./loading-spinner"
import { MapboxMap } from "./mapbox-map"
import { ControlSidebar } from "./control-sidebar"

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
    const [spinner, setSpinner] = useState(true)

    const cityLocation = CITY_LOCATION[currentCity]
    const setCityFromPill = (cityName: string) => {
        setCurrentCity(cityName)
        setCurrentStartingLoc(CITY_LOCATION[cityName])
    }

    return (
        <QueryClientProvider client={queryClient}>
            <LoadingSpinner display={spinner}/>
            <CityPillContainer cities={['Toronto', 'Montreal', 'Vancouver', 'New York City']}
                               setLocation={setCityFromPill} currentCity={currentCity} />
            <MapboxMap currentOptions={currentOptions} currentLatLng={currentStartingLoc}
                       setLatLng={setCurrentStartingLoc}
                       currentPos={cityLocation} setSpinnerLoading={setSpinner} />
            <ControlSidebar setOptions={setCurrentOptions} currentCity={currentCity} />
        </QueryClientProvider>
    )
}
