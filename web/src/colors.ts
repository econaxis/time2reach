import createColorMap from "colormap";
import mapboxgl from "mapbox-gl";

const NSHADES = 300
export const cmap = createColorMap({
    alpha: 0.4,
    colormap: "bluered",
    format: "hex",
    nshades: NSHADES,
});

export function mapper(value) {
    value = 1.1 / (1 + Math.exp(-3 * (2 * value - 1.2))) - 0.05
    return value
}

window.mapper = mapper;


export function get_color_0_1(value: number): string {
    if (value < 0 || value > 1) {
        console.log('invalid value', value)
    }

    value = Math.sqrt(value)
    value = window.mapper(value)
    return cmap[Math.trunc(value * NSHADES)]
}
export class TimeColorMapper {
    m: Record<number, any>;
    raw: Record<number, any>;
    min: number;
    max: number;
    request_id: number;

    constructor() {
        this.m = {};
        this.min = 9999999999999;
        this.max = -this.min;
        this.raw = {}
        this.request_id = 0;
    }

    static async fetch(latlng: mapboxgl.LngLat) {
        const body = {
            latitude: latlng.lat,
            longitude: latlng.lng,
            agencies: ["TTC"]
        }
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

    get_color(from_node: number, to_node: number): string {
        let time;
        if (this.m[from_node] && this.m[to_node]) {
            time =
                (this.m[from_node].timestamp + this.m[to_node].timestamp) /
                2;
        } else {
            time = undefined;
        }

        if (time === undefined) {
            return "#A382821C";
        } else {
            let time_mapped =
                (time - this.min) /
                (this.max - this.min);
            time_mapped = Math.round(time_mapped * 100);

            time_mapped = Math.min(99, time_mapped);
            time_mapped = Math.max(0, time_mapped);
            // Add to make alpha less
            return cmap[time_mapped] + "CC";
        }
    }
}