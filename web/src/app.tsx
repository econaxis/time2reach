import { useEffect, useRef, useState } from "preact/hooks";
import mapboxgl from "mapbox-gl";

interface Agency {
    agencyCode: string;
    agencyLongName: string;
}

export function AgencyEntry({ agencyCode, agencyLongName }: Agency) {
    // agencyCode: TTC/YRT/UP ...
    // agencyLongName: Toronto Transit Commission

    const id = `agency-${agencyCode}`;
    return (
        <div>
            <input
                id={id}
                type="checkbox"
                checked="checked"
                className="checkbox"
            />
            <label htmlFor={id} className="ml-1 text-gray-900">
                {agencyLongName}
            </label>
        </div>
    );
}

export function Header({ children }) {
    return (
        <h2 className="font-medium text-lg font-bold border-b mt-3">
            {children}
        </h2>
    );
}

export function AgencyForm({ agencies, header }) {
    const agencyList = agencies.map((ag) => (
        <AgencyEntry {...ag}></AgencyEntry>
    ));
    return (
        <div>
            <Header>{header}</Header>

            <form id="agency-form" className="mt-2">
                {agencyList}
            </form>
        </div>
    );
}

export function Sidebar({ children }) {
    return (
        <div className="absolute top-0 right-0 m-5 max-w-sm p-5 bg-white border border-gray-200 rounded-lg shadow">
            <p className="text-gray-700">
                Double click anywhere to see how far you can go by public
                transit.
            </p>

            {children}
        </div>
    );
}

const TORONTO_AGENCIES = [
    {
        agencyCode: "TTC",
        agencyLongName: "Toronto Transit Commission",
    },
    {
        agencyCode: "YRT",
        agencyLongName: "York Region Transit",
    },
];

const MODES = [
    { agencyCode: "Bus", agencyLongName: "Bus" },
    { agencyCode: "Subway", agencyLongName: "Subway" },
    { agencyCode: "Tram", agencyLongName: "Tram" },
    { agencyCode: "Train", agencyLongName: "Train" },
];

export function MapboxMap() {
    const map = useRef<mapboxgl.Map | null>(null);
    const mapContainer = useRef<any>(null);
    useEffect(() => {
        // Init mapbox gl map here.
        if (map.current) return;

        mapboxgl.accessToken =
            "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A";

        map.current = new mapboxgl.Map({
            container: mapContainer.current, // container ID
            style: "mapbox://styles/mapbox/dark-v11", // style URL
            center: [-79.43113401487446, 43.650685085905365], // starting position [lng, lat]
            zoom: 12, // starting zoom
        });
    });

    return <div ref={mapContainer} className="map w-screen h-screen"></div>;
}

export function TimeSlider() {
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
                    value="3600"
                    className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                />
            </div>
        </div>
    );
}
export function App() {
    return (
        <div>
            <MapboxMap></MapboxMap>
            <Sidebar>
                <AgencyForm
                    agencies={TORONTO_AGENCIES}
                    header="Agencies"
                ></AgencyForm>

                <AgencyForm agencies={MODES} header="Modes"></AgencyForm>

                <TimeSlider></TimeSlider>
            </Sidebar>
        </div>
    );
}
