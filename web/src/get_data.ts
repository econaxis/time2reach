import { type TimeColorMapper } from "./colors";
import { type LngLat } from "mapbox-gl";
import { baseUrl } from "./dev-api";
import { type TripDetailsTransit } from "@/format-details";

export interface DetailResponse {
    details: TripDetailsTransit[]
    path: GeoJSON.Feature
}
export async function getDetails(data: TimeColorMapper, location: LngLat, signal: AbortSignal): Promise<DetailResponse> {
    const body = {
        request_id: data.request_id,
        latlng: {
            latitude: location.lat,
            longitude: location.lng,
        },
    };
    const resp = await fetch(`${baseUrl}/details/`, {
        method: "POST",
        mode: "cors",
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
        signal
    });

    return await resp.json();
}
