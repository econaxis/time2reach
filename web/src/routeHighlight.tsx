import { useEffect } from 'react';
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

function toLatLng(point: number[]): mapboxgl.LngLat {
    return new mapboxgl.LngLat(point[0], point[1]);
}
export default function RouteHighlight(props: RouteHighlightProps) {
    const { map, routeData } = props;

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

            const projectedPoint = map.project(toLatLng(point));
            const projectedMouse = e.point;


            // Check if the distance is within 10 meters
            if (projectedPoint.dist(projectedMouse) <= 25) {
                console.log('Hello');
                props.setHighlightedPoints({ geojson_index: nearestParams.properties.index });
            }
        };

        // Add the mousemove event listener to the map
        map.on('mousemove', mousemoveHandler);

        // Clean up the event listener when the component unmounts
        return () => {
            map.off('mousemove', mousemoveHandler);
        };
    }, [map, routeData]);

    return null; // Assuming this is a functional component and doesn't render anything
}
