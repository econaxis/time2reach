import type { useState } from "react";
import { SearchBox } from "@mapbox/search-js-react";
import type { OrgDest } from "@/bike";
import mapboxgl from "mapbox-gl";
import { FiArrowDown, FiArrowRight } from "react-icons/fi";
export interface SearchBoxComponentProps {
    currentLocation: mapboxgl.LngLat
    onChange: (pos: mapboxgl.LngLat) => void
    placeholder?: string
    className?: string
}

export type T = ReturnType<typeof useState<OrgDest>>[1];
export interface ToFromSearchBoxProps {
    handleSetLocation: (fn: (orgDest: OrgDest) => OrgDest) => void
    currentLocation: mapboxgl.LngLat
}

function SearchBoxComponent(props: SearchBoxComponentProps) {
    const baseClass = "flex-grow flex-shrink ";
    const className = baseClass + (props.className ?? '');
    return (
        <form className={className}>
            {/* @ts-expect-error Searchbox is not a valid component */}
            <SearchBox
                options={{
                    proximity: props.currentLocation,
                    country: "US",
                }}
                placeholder={props.placeholder ?? "Enter a location"}
                value={""}
                accessToken={
                    "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A"
                }
                onRetrieve={(results) => {
                    const lngLat = results.features[0].geometry.coordinates;
                    props.onChange(new mapboxgl.LngLat(lngLat[0], lngLat[1]));
                }}
            />
        </form>
    );
}

export function ToFromSearchBox(props: ToFromSearchBoxProps) {
    const setOrigin = (pos: mapboxgl.LngLat) => {
        props.handleSetLocation((orgDest) => {
            return {
                origin: pos,
                destination: orgDest?.destination
            };
        });
    };

    const setDestination = (pos: mapboxgl.LngLat) => {
        props.handleSetLocation((orgDest) => {
            return {
                origin: orgDest?.origin,
                destination: pos
            };
        });
    };

    return (
        <div className="absolute top-0 left-0 p-5 text-card-foreground flex sm:flex-row flex-col items-center gap-0 sm:gap-4 w-full sm:max-w-none max-w-40">
            <SearchBoxComponent
                currentLocation={props.currentLocation}
                onChange={setOrigin}
                placeholder="Starting location"
                className="sm:max-w-xs w-full mb-0.5 sm:mb-0" // Adjusted bottom margin for mobile
            />
            <div className="sm:hidden">
                <FiArrowDown size="24" className="flex-none" />
            </div>
            <div className="hidden sm:flex">
                <FiArrowRight size="24" className="flex-none" />
            </div>
            <SearchBoxComponent
                onChange={setDestination}
                currentLocation={props.currentLocation}
                placeholder="Destination"
                className="sm:max-w-xs w-full mt-0.5 sm:mt-0" // Adjusted top margin for mobile
            />
        </div>
    );
}
