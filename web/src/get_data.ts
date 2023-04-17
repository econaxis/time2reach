import { TimeColorMapper } from "./colors";
import { CITY } from "./ol";

export async function getDetails(data: TimeColorMapper, location: object) {
    const resp = await fetch(
        `http://localhost:3030/details/${CITY}/${data.request_id}`,
        {
            method: "POST",
            mode: "cors",
            headers: {
                Accept: "application/json",
                "Content-Type": "application/json",
            },
            body: JSON.stringify(location),
        }
    );

    return resp.json();
}
