import React from "react";
import Chart from "chart.js/auto";
import { Line } from "react-chartjs-2";

import { BellRing, Check } from "lucide-react"

import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
} from "@/components/ui/card"
import { Switch } from "@/components/ui/switch"

interface LineGraphProps {
    data: number[]
}

// @ts-expect-error unused but it's fine because we need to import Chart to work
function _unused() {
    return Chart.length + 1;
}

const notifications = [
    {
        title: "Your call has been confirmed.",
        description: "1 hour ago",
    },
    {
        title: "You have a new message!",
        description: "1 hour ago",
    },
    {
        title: "Your subscription is expiring soon!",
        description: "2 hours ago",
    },
]

type CardProps = React.ComponentProps<typeof Card>

export function CardDemo({ className, ...props }: CardProps) {
    return (
        <Card className={cn("w-[380px]", className)} {...props}>
            <CardHeader>
                <CardTitle>Notifications</CardTitle>
                <CardDescription>You have 3 unread messages.</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4">
                <div className=" flex items-center space-x-4 rounded-md border p-4">
                    <BellRing />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            Push Notifications
                        </p>
                        <p className="text-sm text-muted-foreground">
                            Send notifications to device.
                        </p>
                    </div>
                    <Switch />
                </div>
                <div>
                    {notifications.map((notification, index) => (
                        <div
                            key={index}
                            className="mb-4 grid grid-cols-[25px_1fr] items-start pb-4 last:mb-0 last:pb-0"
                        >
                            <span className="flex h-2 w-2 translate-y-1 rounded-full bg-sky-500" />
                            <div className="space-y-1">
                                <p className="text-sm font-medium leading-none">
                                    {notification.title}
                                </p>
                                <p className="text-sm text-muted-foreground">
                                    {notification.description}
                                </p>
                            </div>
                        </div>
                    ))}
                </div>
            </CardContent>
            <CardFooter>
                <Button className="w-full">
                    <Check className="mr-2 h-4 w-4" /> Mark all as read
                </Button>
            </CardFooter>
        </Card>
    )
}

const ElevationChart: React.FC<LineGraphProps> = ({ data }) => {
    const chartData = {
        labels: Array.from({ length: data.length }, (_, index) => index + 1),
        datasets: [
            {
                label: "",
                data,
                borderColor: "rgba(75,192,192,1)",
                borderWidth: 1,
                radius: 0,
            },
        ],
    };

    const options = { plugins: { legend: { display: false } }, interaction: { intersect: false } };
    return (
        <div style={{ width: 300, height: 600 }}>
            <h2>Floating Point Numbers Line Graph</h2>
            <Line data={chartData} options={options} />
            <CardDemo/>
        </div>
    );
};

export default ElevationChart;
