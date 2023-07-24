import createColorMap from "colormap";
import type mapboxgl from "mapbox-gl";
import { baseUrl } from "./dev-api";
import { BG_WHITE_COLOR } from "./app";
import { formatDuration } from "./format-details";
import { Header } from "./control-sidebar";
import { type ReactNode, useRef } from "react";

function generateCmap(shades: number): string[] {
    const cmap = createColorMap({
        alpha: 0.4,
        colormap: "temperature",
        format: "hex",
        nshades: shades * 2,
    }).reverse();

    return cmap.filter((_value, index) => {
        return index % 2 === 0;
    });

    // const endFirstSlope = 8;
    // const SHADES = 120;
    // const firstSlope = 0.7 * SHADES / shades;
    // // const cmap = createColorMap({
    // //     alpha: 0.4,
    // //     colormap: "temperature",
    // //     format: "hex",
    // //     nshades: SHADES + 1,
    // // });
    //
    // const at = (index) => {
    //     return cmap[cmap.length - Math.round(index) - 1]
    // };
    //
    // const answer: string[] = [];
    // let currentY = 0;
    // for (let i = 0; i < endFirstSlope; i++) {
    //     currentY = i * firstSlope * SHADES / shades;
    //     answer.push(at(currentY));
    // }
    //
    // const secondSlope = (SHADES - currentY) / (shades - endFirstSlope);
    //
    // while (answer.length < shades) {
    //     currentY += secondSlope;
    //     answer.push(at(currentY));
    // }
    // return answer;
}

const NSHADES = 7;
export const cmap = generateCmap(NSHADES);

export function getColor0To1(value: number): string {
    if (value < 0 || value > 1) {
        return "rgba(72,31,2,0.49)";
    }

    let index = Math.trunc(value * NSHADES);

    if (index >= cmap.length) index = cmap.length - 1;
    else if (index < 0) index = 0;

    return cmap[index];
}

function objectToTrueValues(obj: Record<string, boolean>): string[] {
    return Object.entries(obj)
        .filter(([_, include]) => include)
        .map(([key, _include]) => key);
}
export class TimeColorMapper {
    m: Record<number, any>;
    raw: Record<number, any>;
    min: number;
    max: number;
    request_id: any;

    constructor(
        requestId: object,
        edgeTimes: Record<string, number>,
        durationRange: number,
        minDuration: number
    ) {
        this.m = {};
        this.min = Number.MAX_SAFE_INTEGER;
        this.max = Number.MIN_SAFE_INTEGER;
        this.raw = {};
        this.request_id = 0;

        for (const nodeid in edgeTimes) {
            this.raw[nodeid.toString()] = edgeTimes[nodeid];
            const time = edgeTimes[nodeid];
            this.min = Math.min(this.min, time);
        }
        this.request_id = requestId;

        this.max = this.min + durationRange;

        this.min += minDuration;
        this.calculate_colors();
    }

    static async fetch(
        location: mapboxgl.LngLat,
        startTime: number,
        durationRange: number,
        agencies: Record<string, boolean>,
        modes: Record<string, boolean>,
        minDuration: number
    ): Promise<TimeColorMapper> {
        const body = {
            latitude: location.lat,
            longitude: location.lng,
            agencies: objectToTrueValues(agencies),
            modes: objectToTrueValues(modes),
            startTime,
            maxSearchTime: durationRange,
        };

        const data = await fetch(`${baseUrl}/hello/`, {
            method: "POST",
            mode: "cors",
            headers: {
                Accept: "application/json",
                "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
        });

        if (data.ok) {
            const js = await data.json();

            const { request_id: requestId, edge_times: edgeTimes } = js;

            return new TimeColorMapper(requestId, edgeTimes, durationRange, minDuration);
        } else {
            const text = await data.text();
            if (!text.includes("Invalid city")) {
                console.error("Unexpected error from API: ", data, text);
            }

            throw Error("API returned error response" + JSON.stringify(data) + " " + text);
        }
    }

    calculate_colors(): void {
        const spread = this.max - this.min;

        for (const id in this.raw) {
            let normalized = this.raw[id] - this.min;
            normalized /= spread;

            if (normalized <= 1.0) {
                const color = getColor0To1(normalized);
                if (color) {
                    this.m[id] = color;
                } else {
                    console.log("err color", color, normalized);
                }
            }
        }
    }
}

export interface ColorLegendProps {
    tcm: TimeColorMapper;
    currentHover?: number;
}

export interface TickTriangleProps {
    lpercentage: number;
}

export interface TickProps extends TickTriangleProps {
    noRotate?: boolean;
    children: ReactNode;
}
function Tick({ noRotate, children, lpercentage }: TickProps) {
    const color = "rgb(38,38,38)";
    return (
        <div className="absolute left-0 inline-block" style={{ left: `${lpercentage}%` }}>
            <span
                className="inline-block text-xxs font-extralight"
                style={{
                    transform: noRotate ? "" : "translate(-40%, 0%)",
                }}
            >
                {children}
            </span>
            <svg width="2.0" height="4">
                <rect x="0" y="0" width="0.5" height="4" fill={color} />
            </svg>
        </div>
    );
}

function TickTriangle({ lpercentage }: TickTriangleProps) {
    return (
        <div
            className="absolute left-0 inline-block"
            style={{
                left: `${lpercentage}%`,
                transform: "translateY(7px)",
                transition: "all 600ms ease",
                transitionProperty: "left",
            }}
        >
            <span className="inline-block text-lg font-extralight">▼</span>
        </div>
    );
}

export function ColorLegend({ tcm, currentHover }: ColorLegendProps) {
    const lastTick = useRef<any>(null);

    const numSteps = 10;
    const cssGradient: string[] = [];

    for (let i = 0; i <= numSteps; i++) {
        const fraction = i / numSteps;
        const color = getColor0To1(fraction);
        cssGradient.push(`${color} ${(fraction * 100).toFixed(1)}%`);
    }

    const spread = tcm.max - tcm.min;

    const ticks: any[] = [];
    for (let i = 0; i <= spread + 1; i += 3600 * 0.25) {
        const percentage = Math.round((i / spread) * 100);
        const duration = formatDuration(i);
        const cleaned = duration.substring(1, 5);
        ticks.push(
            <Tick key={i.toFixed(0)} lpercentage={percentage}>
                {cleaned}
            </Tick>
        );
    }

    if (currentHover) {
        lastTick.current = (
            <TickTriangle key={"hover"} lpercentage={((currentHover - tcm.min) / spread) * 100} />
        );
        ticks.push(lastTick.current);
    } else if (lastTick.current) {
        ticks.push(lastTick.current);
    }

    const cssStyle = "linear-gradient(to right," + cssGradient.join(",") + ")";
    return (
        <div
            className={`hidden sm:block ${BG_WHITE_COLOR} absolute bottom-0 l-0 m-4 z-50 w-60 max-w-sm lg:max-w-md pb-3 pr-4 pl-5 rounded-lg`}
        >
            <Header>Trip Duration</Header>
            <div
                className="w-full m-auto mt-1 md:mt-2 relative left-0 top-0"
                style={{ height: "1.7rem", left: "2px" }}
            >
                {ticks}
            </div>
            <div
                className="rounded-md w-full m-auto"
                style={{ background: cssStyle, height: "1.5rem" }}
            ></div>
        </div>
    );
}
