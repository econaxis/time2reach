import { get_color_0_1, TimeColorMapper } from "./colors";

export async function get_data(body): Promise<TimeColorMapper> {
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

    const colors = new TimeColorMapper();

    for (const nodeid in js) {
        colors.m[nodeid.toString()] = js[nodeid]
    }
    console.log(colors.m)
    for (const nodeid in js) {
        const time = js[nodeid];
        colors.min = Math.min(colors.min, time);
        colors.max = Math.max(colors.max, time);
    }

    const spread = colors.max - colors.min

    for (const id in colors.m) {
        colors.m[id] -= colors.min
        colors.m[id] /= spread

        colors.m[id] = get_color_0_1(colors.m[id])

    }
    window.colors = colors.m;
    return colors;
}