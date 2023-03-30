import { get_color_0_1, TimeColorMapper } from "./colors";

export async function get_data(body): Promise<TimeColorMapper> {
    console.log('getting new data', body)
    const data = await fetch("http://localhost:3030/hello", {
        method: "POST",
        mode: "cors",
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        },
        body: JSON.stringify(body)
    });
    const js = await data.json();

    const {request_id, edge_times} = js;

    const colors = new TimeColorMapper();

    for (const nodeid in edge_times) {
        colors.raw[nodeid.toString()] = edge_times[nodeid]
        const time = edge_times[nodeid];
        colors.min = Math.min(colors.min, time);
        colors.max = Math.max(colors.max, time);
    }

    const spread = colors.max - colors.min

    for (const id in colors.raw) {
        colors.m[id] = colors.raw[id] - colors.min
        colors.m[id] /= spread

        colors.m[id] = get_color_0_1(colors.m[id])

    }
    colors.request_id = request_id;
    window.colors = colors.m;
    return colors;
}

export async function get_details(data: TimeColorMapper, location: object) {
    const resp = await fetch(`http://localhost:3030/details/${data.request_id}`, {
        method: "POST",
        mode: "cors",
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        },
        body: JSON.stringify(location)
    });

    return resp.json();
}