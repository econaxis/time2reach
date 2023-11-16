import React from "react";
import Chart from "chart.js/auto";
import { Line } from "react-chartjs-2";

import { BellRing, Check } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";

interface LineGraphProps {
    data: number[]
}

// @ts-expect-error unused but it's fine because we need to import Chart to work
function _unused() {
    return Chart.length + 1;
}

export default function ElevationChart({ data }) {
    if (!data) {
        return <></>;
    }

    const chartData = {
        datasets: [
            {
                label: "",
                data: data.map((a) => ({ x: a[0], y: a[1] })),
                borderColor: "rgba(75,192,192,1)",
                borderWidth: 1,
                radius: 0,
                fill: { target: "origin", above: "rgba(75,192,192,0.4)" },
            },
            {
                label: "",
                data: [{ x: 0, y: data[data.length - 1][1] }],
                yAxisID: "y1",
            }
        ],
    };

    const maxRight = data[data.length - 1][1] + 1;
    const maxLeft = Math.max(...data.map((a) => a[1]));

    const maxTotal = Math.max(maxRight, maxLeft);
    const distance = Math.round(data[data.length - 1][0]);
    const useKilometers = distance > 4000;
    const options = {
        scales: {
            y1: {
                grid: { drawTicks: false },
                min: 0,
                max: maxTotal,
                ticks: {
                    stepSize: 1,
                    autoSkip: false,
                    callback: (value, index, values) => {
                        if (value === Math.round(data[data.length - 1][1])) return value.toString();
                        // else if (value === Math.round(data[data.length - 1][1])) { return value.toString(); } else return null;
                    },
                },
                type: "linear",
                position: 'right'
            },
            y: {
                grid: { drawTicks: false },
                min: 0,
                max: maxTotal,
                ticks: {
                    stepSize: 1,
                    autoSkip: false,
                    callback: (value, index, values) => {
                        if (index === values.length - 1) {
                            return value.toString();
                        } else if (index === 0) return value.toString();
                        else if (value === Math.round(data[0][1])) return value.toString();
                        // else if (value === Math.round(data[data.length - 1][1])) { return value.toString(); } else return null;
                    },
                },
            },
            x: {
                grid: { drawTicks: false },
                min: 0,
                max: Math.round(Math.max(...data.map((a) => a[0]))),
                ticks: {
                    display: true,
                    autoSkip: false,
                    callback: (value, index, values) => {
                        if (value !== 0) {
                            if (useKilometers) return (Math.round(value / 100) / 10).toString();
                            else return value.toString();
                        } else return null;
                    },
                },
                type: "linear",
            },
        },
        plugins: { legend: { display: false } },
        interaction: { intersect: false },
    };
    return (
        <Card className="w-[320px] relative z-10">
            <CardHeader className="p-4">
                <CardTitle>Elevation Chart</CardTitle>
            </CardHeader>
            <CardContent className="p-2.5">
                <Line data={chartData} options={options} />
            </CardContent>
        </Card>
    );
}
