import type { useState } from "react";
import { SearchBox } from "@mapbox/search-js-react";
import type { OrgDest } from "@/bike";
import mapboxgl, { type LngLat } from "mapbox-gl";
import { FiArrowDown, FiArrowRight } from "react-icons/fi";
import { type SearchBoxRefType } from "@mapbox/search-js-react/src/components/SearchBox";
import React from "react";

import { useQuery } from 'react-query';
import { useOrgDestContext } from "@/bike";
export interface SearchBoxComponentProps {
    currentLocation?: mapboxgl.LngLat
    onChange: (pos: mapboxgl.LngLat) => void
    defaultLocation: string
    placeholder?: string
    className?: string
}

export type T = ReturnType<typeof useState<OrgDest>>[1];
export interface ToFromSearchBoxProps {
    handleSetLocation: (fn: (orgDest: OrgDest) => OrgDest) => void
    currentLocation: mapboxgl.LngLat
}

function SearchBoxComponent(props: SearchBoxComponentProps) {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars

    const baseClass = "flex-grow flex-shrink ";
    const className = baseClass + (props.className ?? '');

    const sbRef = React.useRef<SearchBoxRefType>();

    // Define the fetch function
    const fetchGeocodedLocation = async(location: LngLat): Promise<string> => {
        const apiUrl = `https://api.mapbox.com/geocoding/v5/mapbox.places/${location.lng},${location.lat}.json?access_token=pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A`;
        const response = await fetch(apiUrl);
        const data = await response.json();
        return data.features[0].place_name;
    };

    // Use the useQuery hook to fetch the location
    const { data: coordinates, isLoading } = useQuery(
        ['geocode', props.currentLocation],
        async() => await fetchGeocodedLocation(props.currentLocation!),
        {
            enabled: !!props.currentLocation,
            keepPreviousData: true, // Keep displaying the old data while loading new data
        }
    );

    return (
        <form className={className}>
            {/* @ts-expect-error fdsa */}
            <SearchBox
                ref={sbRef}
                options={{
                    proximity: props.currentLocation,
                    country: "US",
                }}
                placeholder={isLoading ? "..." : (props.placeholder ?? "Enter a location")}
                value={isLoading ? "..." : coordinates} // Use loading placeholder or a default value
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
    const orgDest = useOrgDestContext()[0];
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

    const searchBoxClassName = " w-full my-1.5 sm:mb-0"
    return (
        <div className="absolute top-0 left-0 right-0 mx-auto p-5 text-card-foreground flex sm:flex-row flex-col items-center gap-0 sm:gap-4  max-w-3xl">
            <SearchBoxComponent
                defaultLocation="222 Lily St"
                currentLocation={orgDest?.origin}
                onChange={setOrigin}
                placeholder="Starting location"
                className={searchBoxClassName} // Adjusted bottom margin for mobile
            />
            <div className="sm:hidden">
                <FiArrowDown size="24" className="flex-none" />
            </div>
            <div className="hidden sm:flex">
                <FiArrowRight size="24" className="flex-none" />
            </div>
            <SearchBoxComponent
                defaultLocation="Presidio"
                onChange={setDestination}
                currentLocation={orgDest?.destination}
                placeholder="Destination"
                className={searchBoxClassName} // Adjusted top margin for mobile
            />
        </div>
    );
}
