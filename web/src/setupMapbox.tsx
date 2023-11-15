import type mapboxgl from "mapbox-gl";
import { Fragment, useEffect } from "react";

export interface SetupProps {
    map: mapboxgl.Map | undefined
    setOrigin: (latLng: mapboxgl.LngLat) => void
    setDestination: (latLng: mapboxgl.LngLat) => void
}

export function SetupMapbox(props: SetupProps) {
    const { map, setOrigin, setDestination } = props;
    useEffect(() => {
        if (!map) {
            return;
        }
        console.log("SETTING UP MAPBOX");
        let isOrg = true;
        const dblClickHandler = (e) => {
            e.preventDefault();
            console.log("Double clicked!", isOrg);

            if (isOrg) {
                setOrigin(e.lngLat);
            } else {
                setDestination(e.lngLat);
            }
            isOrg = !isOrg;
        };
        map.on("dblclick", dblClickHandler);

        return () => {
            map.off("dblclick", dblClickHandler);
        };
    }, [map]);

    return <Fragment />;
}
