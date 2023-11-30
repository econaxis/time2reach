import Chart from "chart.js/auto";
import { type Chart as ChartJS } from "chart.js";
import { Line } from "react-chartjs-2";

import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { type HighlightedPointElev } from "@/routeHighlight";
import { useRef } from "react";

interface LineGraphProps {
    elevationData: number[]
    hp?: HighlightedPointElev
}

// @ts-expect-error unused but it's fine because we need to import Chart to work
function _unused() {
    return Chart.length + 1;
}


export default function ElevationChart({ elevationData, hp }: LineGraphProps) {
    const chartRef = useRef<ChartJS | undefined>()

    if (!elevationData) {
        return <></>;
    }

    const chartData = {
        datasets: [
            {
                label: "",
                data: elevationData.map((a) => ({ x: a[0], y: a[1] })),
                borderColor: "rgba(75,192,192,1)",
                borderWidth: 1,
                radius: 0,
                fill: { target: "origin", above: "rgba(75,192,192,0.4)" },
            },
            {
                label: "",
                data: [{ x: 0, y: elevationData[elevationData.length - 1][1] }],
                yAxisID: "y1",
            }
        ],
    };

    if (hp) {
        const chart = chartRef.current;
        chart.setActiveElements([{
            datasetIndex: 0,
            index: hp.elevation_index,
        }])
        // tooltip.setActiveElements([
        //     {
        //         datasetIndex: 0,
        //         index: hp.elevation_index,
        //     }
        // ], {
        //     x: (chartArea.left + chartArea.right) / 2,
        //     y: (chartArea.top + chartArea.bottom) / 2,
        // });
        chart.update();
    }

    const maxRight = elevationData[elevationData.length - 1][1] as number + 1;
    const maxLeft = Math.max(...elevationData.map((a) => a[1]));

    const maxTotal = Math.max(maxRight, maxLeft);
    const distance = Math.round(elevationData[elevationData.length - 1][0]);
    const useKilometers = distance > 4000;
    const options = {
        animation: true,
        scales: {
            y1: {
                grid: { drawTicks: false },
                min: 0,
                max: maxTotal,
                ticks: {
                    stepSize: 1,
                    autoSkip: false,
                    callback: (value, index, values) => {
                        if (index === values.length - 1) {
                            // Max elevation tick
                            return Math.round(value).toString();
                        } else if (value === Math.round(elevationData[elevationData.length - 1][1])) {
                            // End (destination) tick
                            return value.toString();
                        }
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
                         if (index === 0) {
                             // 0 tick
                            return Math.round(value).toString();
                        } else if (value === Math.round(elevationData[0][1])) {
                             // Start (origin) tick
                            return Math.round(value).toString();
                        }
                    },
                },
            },
            x: {
                grid: { drawTicks: false },
                min: 0,
                max: Math.round(Math.max(...elevationData.map((a) => a[0]))),
                ticks: {
                    display: true,
                    autoSkip: false,
                    callback: (value, _index, _values) => {
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
        elements: {
            point: {
                hoverRadius: 2.2,
                hoverBorderWidth: 4.5
            }
        }
    };
    return (
        <Card className="w-[320px] absolute top-0 left-0 z-10">
            <CardHeader className="p-4">
                <CardTitle>Elevation (meters)</CardTitle>
            </CardHeader>
            <CardContent className="p-2.5">
                <Line ref={chartRef} data={chartData} options={options} />
            </CardContent>
        </Card>
    );
}
