import createColorMap from "colormap";

const NSHADES = 300
export const cmap = createColorMap({
    alpha: 0.4,
    colormap: "bluered",
    format: "hex",
    nshades: NSHADES,
});

export function get_color_0_1(value: number): string {
    if (value < 0 || value > 1) {
        console.log('invalid value', value)
    }

    value = Math.sqrt(value)
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