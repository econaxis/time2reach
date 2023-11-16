import type mapboxgl from "mapbox-gl";
import { Fragment, useEffect } from "react";
import { type OrgDest } from "@/bike";

export interface SetupProps {
    map: mapboxgl.Map | undefined;
    // A function that takes a function that takes in an OrgDest and returns void
    setOrgDest: (f: (orgDest: OrgDest) => OrgDest) => void;
}

export function SetupMapbox(props: SetupProps) {
    const { map, setOrgDest } = props;
    useEffect(() => {
        if (!map) {
            return;
        }
        console.log("SETTING UP MAPBOX");
        const dblClickHandler = (e) => {
            e.preventDefault();

            setOrgDest((orgDest) => {
                return {
                    origin: orgDest.destination,
                    destination: e.lngLat
                };
            });
        };
        map.on("dblclick", dblClickHandler);

        return () => {
            map.off("dblclick", dblClickHandler);
        };
    }, [map]);

    return <Fragment />;
}
