import createColorMap from "colormap";

export const cmap = createColorMap({
    alpha: 0.4,
    colormap: "jet",
    format: "hex",
    nshades: 100,
});

export class TimeColorMapper {
    m: Map<number, number>;
    min: number;
    max: number;

    constructor() {
        this.m = new Map();
        this.min = Infinity;
        this.max = -Infinity;
    }

    get_color(from_node: number, to_node: number): string {
        let time;
        if (this.m[from_node] && this.m[to_node]) {
            time =
                (this.m[from_node] + this.m[to_node]) /
                2;
        } else {
            time = undefined;
        }

        if (time === undefined) {
            return "#A7727244";
        } else {
            let time_mapped =
                (time - this.min) /
                (this.max - this.min);
            time_mapped = Math.round(time_mapped * 100);

            time_mapped = Math.min(99, time_mapped);
            time_mapped = Math.max(0, time_mapped);
            // Add "99" to make alpha less
            return cmap[time_mapped] + "77";
        }
    }
}