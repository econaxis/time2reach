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
import { color } from "chart.js/helpers";
import { hashElevationData } from "@/bike";

interface LineGraphProps {
    elevationDataHistory: number[][][]
    elevationData: number[][]
    hp?: HighlightedPointElev
}

// @ts-expect-error unused but it's fine because we need to import Chart to work
function _unused() {
    return Chart.length + 1;
}

export default function ElevationChart({ elevationData, elevationDataHistory, hp }: LineGraphProps) {
    const chartRef = useRef<ChartJS<"line"> | undefined>()

    useEffect(() => {
        if (hp && chartRef.current && elevationData) {
            const chart = chartRef.current;
            console.log("Setting active elements", hp.elevation_index, chart.data)

            if (hp.elevation_index >= chart.data.datasets[0].data.length) {
                console.warn("Elevation index out of bounds")
                return
            }
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
    }, [hp, chartRef.current]);

    if (!elevationData) {
        return <></>;
    }

    console.log("Currently rendered: ", hashElevationData(elevationData), elevationData.length)
    const chartData = {
        datasets: [
            {
                label: "",
                data: elevationData.map((a) => ({ x: a[0], y: a[1] })),
                borderColor: "rgba(75,192,192,1)",
                borderWidth: 1,
                radius: 0,
                fill: { target: "origin", above: "rgba(45,231,231,0.2)" },
            }
        ],
    };
    for (const [index, history] of elevationDataHistory.entries()) {
        if (index === elevationDataHistory.length - 1) break;
        if (index !== 0) continue;

        const primary = "rgba(97,106,110,0.8)"
        // const alpha = 0.2 + (0.8 / elevationDataHistory.length) * index;
        const alpha = 0.8;
        const secondary = `rgba(177,188,190,${alpha})`
        const color = index === elevationDataHistory.length - 2 ? primary : secondary;

        chartData.datasets.push({
            borderColor: color,
            borderWidth: 0.55,
            data: history.map((a) => ({ x: a[0], y: a[1] })),
            fill: undefined,
            label: "",
            radius: 0,
            yAxisID: "y",
        });
    }

    let elevData1: number[][];
    if (elevationDataHistory.length > 0) {
        elevData1 = elevationDataHistory[0];
    } else {
        elevData1 = elevationData;
    }

    const maxRight = elevData1[elevData1.length - 1][1] + 1;
    const maxLeft = Math.max(...elevData1.map((a) => a[1]));

    const maxTotal = Math.max(maxRight, maxLeft);
    const distance = Math.round(elevData1[elevData1.length - 1][0]);
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
                grid: { drawTicks: false },
                min: 0,
                max: maxTotal,
                ticks: {
                    stepSize: 1,
                    autoSkip: false,
                    callback: (value, index, _values) => {
                         if (index === 0) {
                             // 0 tick
                            return Math.round(value).toString();
                        } else if (value === Math.round(elevData1[0][1])) {
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
