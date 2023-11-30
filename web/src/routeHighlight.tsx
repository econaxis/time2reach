import { Fragment, useEffect, useState } from "react";
import * as turf from '@turf/turf';
import mapboxgl from "mapbox-gl";
import { type LineString } from "geojson"; // Make sure to install the turf library

export interface HighlightedPointGeoJSON {
    geojson_index: number
}

export interface HighlightedPointElev {
    elevation_index: number
}

interface RouteHighlightProps {
    map: mapboxgl.Map | undefined
    routeData: GeoJSON.FeatureCollection<LineString>
    setHighlightedPoints: (_: HighlightedPointGeoJSON) => void
}

function arrayToLatLng(point: number[]): mapboxgl.LngLat {
    return new mapboxgl.LngLat(point[0], point[1]);
}

interface HighlightMarker {
    // Marker to display along the route of where we're calculating the elevation
    point: mapboxgl.Point
}
export default function RouteHighlight(props: RouteHighlightProps) {
    const { map, routeData } = props;

    const [highlightPos, setHighlightPos] = useState<HighlightMarker | undefined>(undefined);

    useEffect(() => {
        if (!map || !routeData) return;

        const lineStringCoordinates = routeData.features[0].geometry.coordinates.map((coord: number[]) =>
            [coord[0], coord[1]]
        );

        // Create a Turf LineString from the coordinates
        const lineString = turf.lineString(lineStringCoordinates);
        const mousemoveHandler = (e: mapboxgl.MapMouseEvent) => {
            const mouseLatLng = e.lngLat;

            const mousePoint = turf.point([mouseLatLng.lng, mouseLatLng.lat]);

            // Calculate the distance between the mouse cursor and the route
            const nearestParams = turf.nearestPointOnLine(lineString, mousePoint, { units: 'meters' });
            const point = nearestParams.geometry.coordinates;

            const projectedPoint = map.project(arrayToLatLng(point));
            const projectedMouse = e.point;


            // Check if the distance is within 10 meters
            if (projectedPoint.dist(projectedMouse) <= 25) {
                setHighlightPos({
                    point: projectedPoint
                })
                props.setHighlightedPoints({ geojson_index: nearestParams.properties.index });
            } else {
                setHighlightPos(undefined);
            }
        };

        // Add the mousemove event listener to the map
        map.on('mousemove', mousemoveHandler);

        // Clean up the event listener when the component unmounts
        return () => {
            map.off('mousemove', mousemoveHandler);
        };
    }, [map, routeData]);

    if (highlightPos) {
        return <div className="z-10 absolute" style={{
            left: highlightPos.point.x,
            top: highlightPos.point.y,
            transform: "translate(-50%, -50%)",
            borderRadius: "50%",
            width: "10px",
            height: "10px",
            backgroundColor: "#94daef",
            border: "1.5px solid #e1e9f1",
            outline: "0.8px solid black",
        }}></div>
    }
    return <Fragment></Fragment>;
}
