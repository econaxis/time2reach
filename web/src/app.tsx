import { useState } from "react";
import mapboxgl from "mapbox-gl";

import "./style.css";
import { CityPillContainer } from "./city-pill";
import { QueryClient, QueryClientProvider } from "react-query";
import { ControlSidebar } from "./control-sidebar";
import { BlurBackground, WelcomePopup } from "./welcome-popup";

export const BG_WHITE_COLOR = "bg-slate-50";

const CITY_LOCATION = {
    Toronto: new mapboxgl.LngLat(-79.3832, 43.6532),
    "New York City": new mapboxgl.LngLat(-74.006, 40.7128),
    Montreal: new mapboxgl.LngLat(-73.5674, 45.5019),
    Vancouver: new mapboxgl.LngLat(-123.1207, 49.2827),
    "Kitchener-Waterloo": new mapboxgl.LngLat(-80.4935412978086, 43.45134086953097),
};

export const startingLocation = CITY_LOCATION.Toronto;

export function App() {
    const queryClient = new QueryClient({});

    const [currentStartingLoc, setCurrentStartingLoc] = useState(startingLocation);
    const [currentCity, setCurrentCity] = useState("Toronto");

    const [popupAccepted, setPopupAccepted] = useState(false);

    const setCityFromPill = (cityName: string) => {
        setCurrentCity(cityName);
        setCurrentStartingLoc(CITY_LOCATION[cityName]);
    };

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
                        "Kitchener-Waterloo",
                    ]}
                    setLocation={setCityFromPill}
                    currentCity={currentCity}
                />
                <ControlSidebar defaultStartLoc={currentStartingLoc} currentCity={currentCity} />
            </BlurBackground>
        </QueryClientProvider>
    );
}
