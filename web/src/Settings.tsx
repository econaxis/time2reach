import * as React from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";

export interface SliderProps {
    startValue: number
    onChange: (value: number) => void
    label: string
    hoverDescription: string
    className?: string
}

export function MySlider(props: SliderProps) {
    const [value, setValue] = React.useState(props.startValue);

    const id = props.label.toLowerCase().replace(" ", "-");
    return (
        <div className={cn(props.className, "grid gap-2 pt-2")}>
            <HoverCard openDelay={250}>
                <HoverCardTrigger asChild>
                    <div className="grid gap-3">
                        <div className="flex items-center justify-between">
                            <Label htmlFor={id}>{props.label}</Label>
                            <span className="w-12 px-2 py-0.5 text-right text-sm text-muted-foreground">
                                {value}
                            </span>
                        </div>
                        <Slider
                            id={id}
                            max={1}
                            defaultValue={[value]}
                            step={0.1}
                            onValueChange={(x) => {
                                setValue(x[0])
                                props.onChange(x[0])
                            }}
                        />
                    </div>
                </HoverCardTrigger>
                <HoverCardContent align="start" className="w-[260px] text-sm p-3.5" side="left">
                    {props.hoverDescription}
                </HoverCardContent>
            </HoverCard>
        </div>
    );
}

export interface SettingsProps {
    setAvoidHills: (value: number) => void
    setPreferProtectedLanes: (value: number) => void
}

export default function Settings({ setAvoidHills, setPreferProtectedLanes }: SettingsProps) {
    return <Card className="w-[240px] absolute top-0 right-0 z-10 m-5 p-6 pt-0">
        <MySlider className="mt-5" startValue={0.5} onChange={setAvoidHills} label={"Avoid hills"} hoverDescription={"Increase to prioritize avoiding steep hills (routes will have gradual slopes)"} />
        <MySlider className="mt-5" startValue={0.5} onChange={setPreferProtectedLanes} label={"Prefer protected bike lanes"} hoverDescription={"Increase to prioritize routes that use bike lanes."} />
    </Card>
}
