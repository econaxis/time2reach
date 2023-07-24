import React, { useEffect, useState } from "react";

import mapboxgl from "mapbox-gl";

import "./style.css";
import { CityPillContainer } from "./city-pill";
import { QueryClient, QueryClientProvider } from "react-query";
import { ControlSidebar } from "./control-sidebar";
import { BlurBackground, WelcomePopup } from "./welcome-popup";
import { InformationIcon } from "./information-icon";
import { type BrightnessContextInt } from "./time-slider";

export const BG_WHITE_COLOR = "bg-zinc-50";

export const BrightnessContext = React.createContext<BrightnessContextInt>({
    brightness: 100,
    setBrightness: (_value: number) => {},
});

export const CITY_LOCATION = {
    Toronto: new mapboxgl.LngLat(-79.37988, 43.644622),
    "New York City": new mapboxgl.LngLat(-74.006, 40.7128),
    Montreal: new mapboxgl.LngLat(-73.5674, 45.5019),
    Vancouver: new mapboxgl.LngLat(-123.1207, 49.2827),
    "Kitchener-Waterloo": new mapboxgl.LngLat(-80.4935412978086, 43.45134086953097),
    Paris: new mapboxgl.LngLat(2.3522, 48.8566),
    "San Francisco": new mapboxgl.LngLat(-122.4194, 37.7749),
};

export function MapboxGLCanvasBrightnessHack({ brightness }: { brightness: number }) {
    const element1 = [...document.getElementsByClassName('mapboxgl-canvas')] as HTMLCanvasElement[];
    useEffect(() => {
        // Skip first element as that is the default map layer
        for (const element of element1) {
            if (element?.style) element.style.filter = `brightness(${brightness}%)`;
        }
    }, [brightness, element1])

    return <></>;
}

export function App() {
    const queryClient = new QueryClient({});

    const DEFAULT_CITY = "Paris";

    const [currentCity, setCurrentCity] = useState(DEFAULT_CITY);
    const [currentStartingLoc, setCurrentStartingLoc] = useState(CITY_LOCATION[DEFAULT_CITY]);

    const [popupAccepted, setPopupAccepted] = useState(false);
    const [brightness, setBrightness] = useState(145);

    const setCityFromPill = (cityName: string) => {
        setCurrentCity(cityName);
        setCurrentStartingLoc(CITY_LOCATION[cityName]);
    };

    const brightnessCtx = { brightness, setBrightness }
    return (
        <QueryClientProvider client={queryClient}>
            {popupAccepted ? null : <WelcomePopup acceptedPopupCallback={setPopupAccepted} />}
            <BlurBackground enabled={!popupAccepted}>
                <CityPillContainer
                    cities={[
                        "Toronto",
                        "Montreal",
                        "Vancouver",
                        "New York City",
                        // "Kitchener-Waterloo",
                        "San Francisco",
                        "Paris"
                    ]}
                    setLocation={setCityFromPill}
                    currentCity={currentCity}
                />
                <BrightnessContext.Provider value={brightnessCtx}>
                    <MapboxGLCanvasBrightnessHack brightness={brightness} />
                    <ControlSidebar defaultStartLoc={currentStartingLoc} currentCity={currentCity} />
                </BrightnessContext.Provider>
                <InformationIcon
                    onClick={() => {
                        setPopupAccepted(false);
                    }}
                />
            </BlurBackground>
        </QueryClientProvider>
    );
}
