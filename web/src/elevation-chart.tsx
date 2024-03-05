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
import { useEffect, useRef } from "react";
import type { ElevationChartData } from "@/bike";

interface LineGraphProps {
    elevationData?: ElevationChartData
    highlightedPoint?: HighlightedPointElev
}

// @ts-expect-error unused but it's fine because we need to import Chart to work
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function _unused() {
    return Chart.length + 1;
}

export default function ElevationChart({ elevationData, highlightedPoint, className }: LineGraphProps & { className?: string }) {
    const chartRef = useRef<ChartJS<"line"> | undefined>()

    useEffect(() => {
        if (highlightedPoint && chartRef.current && elevationData) {
            const chart = chartRef.current;
            console.log("Setting active elements", highlightedPoint.elevation_index, chart.data)

            if (highlightedPoint.elevation_index >= chart.data.datasets[0].data.length) {
                console.warn("Elevation index out of bounds")
                return
            }
            chart.setActiveElements([{
                datasetIndex: 0,
                index: highlightedPoint.elevation_index,
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
    }, [highlightedPoint, chartRef.current]);

    if (!elevationData) {
        return <></>;
    }

    console.log("Currently rendering", elevationData)
    const datasets: object[] = [];
    const borderColors = {
        primary: 'rgba(75,192,192,1)',
        secondary: 'rgba(98,97,97,0.67)'
    }
    const fillColors = {
        primary: 'rgba(121,231,231,0.5)',
        secondary: 'rgba(231,226,226,0.3)'
    }

    const foregroundStatus: "primary" | "secondary" = "primary";
    let backgroundStatus: "primary" | "secondary" = "secondary";
    if (!elevationData.foreground) {
        backgroundStatus = "primary";
    }
    if (elevationData.background) {
        datasets.push({
            borderColor: borderColors[backgroundStatus],
            borderWidth: 0.55,
            data: elevationData.background.map((a) => ({ x: a[0], y: a[1] })),
            fill: { target: "origin", above: fillColors[backgroundStatus] },
            label: "",
            radius: 0,
            yAxisID: "y",
        });
    }
    datasets.push({
        borderColor: borderColors[foregroundStatus],
        borderWidth: 0.55,
        data: elevationData.foreground.map((a) => ({ x: a[0], y: a[1] })),
        fill: { target: "origin", above: fillColors[foregroundStatus] },
        label: "",
        radius: 0,
        yAxisID: "y",
    });

    let elevDataForAxes: number[][] = elevationData.foreground;
    if (elevationData.background && elevationData.background.length > elevationData.foreground.length) {
        elevDataForAxes = elevationData.background;
    }

    const maxTotal = elevationData.maxElevation + 20;

    const distance = Math.round(elevDataForAxes[elevDataForAxes.length - 1][0]);
    const useKilometers = distance > 4000;
    const options = {
        animation: false as false,
        scales: {
            // y1: {
            //     grid: { drawTicks: false },
            //     min: 0,
            //     max: maxTotal,
            //     ticks: {
            //         stepSize: 1,
            //         autoSkip: false,
            //         callback: (value, index, values) => {
            //             if (index === values.length - 1) {
            //                 // Max elevation tick
            //                 return Math.round(value).toString();
            //             } else if (value === Math.round(elevationData[elevationData.length - 1][1])) {
            //                 // End (destination) tick
            //                 return value.toString();
            //             }
            //             // else if (value === Math.round(data[data.length - 1][1])) { return value.toString(); } else return null;
            //         },
            //     },
            //     type: "linear",
            //     position: 'right'
            // },
            y: {
                display: false,
                grid: { drawTicks: false },
                min: 0,
                max: maxTotal,
                // ticks: {
                //     stepSize: 1,
                //     autoSkip: false,
                //     callback: (value, index, _values) => {
                //          if (index === 0) {
                //              // 0 tick
                //             return Math.round(value).toString();
                //         } else if (value === Math.round(elevDataForAxes[0][1])) {
                //              // Start (origin) tick
                //             return Math.round(value).toString();
                //         }
                //     },
                // },
            },
            x: {
                display: false,
                grid: { drawTicks: false },
                min: 0,
                max: Math.round(elevationData.maxDistance),
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
        plugins: {
             legend: { display: false },
            tooltip: { enabled: true }
        },
        interaction: { intersect: false },
        elements: {
            point: {
                hoverRadius: 2.2,
                hoverBorderWidth: 4.5
            }
        },
    };

    const chartData = {
        datasets,
    }
    return (
        <div className={`elevation-chart ${className ?? ''}`}>
            {/* @ts-expect-error chartRef is a ref */}
            <Line ref={chartRef} data={chartData} options={options} />
            <h1 className="ml-2 text-sm">Elevation (meters)</h1>
        </div>
    );
}
