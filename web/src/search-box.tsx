import type { useState } from "react";
import { SearchBox } from "@mapbox/search-js-react";
import type { OrgDest } from "@/bike";
import mapboxgl from "mapbox-gl";

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
    const className = "w-56 " + (props.className ?? '')
    return (
        <form
            className={className}
        >
            {/* @ts-expect-error SearchBox cannot be used as a component */}
            <SearchBox
                options={
                    {
                        proximity: props.currentLocation,
                        country: "US",
                    }
                }
                placeholder={props.placeholder ?? "Enter a location"}
                value={""}
                accessToken={
                    "pk.eyJ1IjoiaGVucnkyODMzIiwiYSI6ImNsZjhxM2lhczF4OHgzc3BxdG54MHU4eGMifQ.LpZVW1YPKfvrVgmBbEqh4A"
                }
                onRetrieve={(results) => {
                    const lngLat = results.features[0].geometry.coordinates
                    props.onChange(new mapboxgl.LngLat(lngLat[0], lngLat[1]))
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
            }
        })
    }

    const setDestination = (pos: mapboxgl.LngLat) => {
        props.handleSetLocation((orgDest) => {
            return {
                origin: orgDest?.origin,
                destination: pos
            }
        })
    }
    return (
            <div className="flex flex-col space-y-4 top-0 left-0 absolute m-5 text-card-foreground">
                <SearchBoxComponent
                    currentLocation={props.currentLocation}
                    onChange={setOrigin}
                    placeholder={"Starting location"}
                />
                <SearchBoxComponent
                    className={"mt-5"}
                    onChange={setDestination}
                    currentLocation={props.currentLocation}
                    placeholder={"Destination"}
                />
            </div>
    );
}
