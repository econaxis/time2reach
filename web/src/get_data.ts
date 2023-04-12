import { TimeColorMapper } from "./colors";

export async function get_details(data: TimeColorMapper, location: object) {
    const resp = await fetch(
        `http://localhost:3030/details/${data.request_id}`,
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
