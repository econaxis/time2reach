import React, { useEffect, useState } from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export interface SliderProps {
    startValue: number
    onChange: (value: number, commit: boolean) => void
    label: string
    hoverDescription: string
    className?: string
}

export function MySlider(props: SliderProps) {
    const [value, setValue] = React.useState(props.startValue);
    const [isMobile, setIsMobile] = useState(window.innerWidth < 768);

    useEffect(() => {
        function handleResize() {
            setIsMobile(window.innerWidth < 768);
        }

        window.addEventListener('resize', handleResize);
        return () => { window.removeEventListener('resize', handleResize); };
    }, []);

    const id = props.label.toLowerCase().replace(" ", "-");
    return (
        <div className={cn(props.className, "grid gap-2 pt-1")}>
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
                                setValue(x[0]);
                                props.onChange(x[0], false);
                            }}
                            onValueCommit={(x) => {
                                setValue(x[0]);
                                props.onChange(x[0], true);
                            }}
                        />
                    </div>
                </HoverCardTrigger>
                <HoverCardContent align="start" className="w-[260px] text-sm p-3" side={isMobile ? "bottom" : "left"}>
                    {props.hoverDescription}
                </HoverCardContent>
            </HoverCard>
        </div>
    );
}

function GeneralLabel({ label, value, hover }: { label: string, value: number, hover?: string }) {
    return (
        <HoverCard openDelay={250}>
            <HoverCardTrigger asChild>
                <div className=" flex items-center justify-between">
                    <Label>{label}</Label>
                    <span className="w-12 px-2 py-0.5 text-right text-sm text-muted-foreground">
                        {value}
                    </span>
                </div>
            </HoverCardTrigger>
            {hover && <HoverCardContent className="w-[260px] text-sm p-3" side="left">
                {hover}
            </HoverCardContent>}
        </HoverCard>
    );
}
export function CaloriesCounter({ energy }: RouteInformation) {
    // eslint-disable-next-line @typescript-eslint/naming-convention
    const { calories, uphill_meters, downhill_meters, total_meters } = energy;

    let units = "meters";
    let distance = total_meters;
    if (total_meters > 10000) {
        distance = total_meters / 1000;
        units = "km";
    }
    return (
        <>
        <hr className="mt-2" />
        <GeneralLabel
            label={`Distance (${units})`}
            value={Math.round(distance)}/>
        <GeneralLabel
            label="Calories consumed"
            value={Math.round(calories)}
            hover={`Rough estimate of calories consumed based on hills and distance, assuming 86 kg rider + bike mass.`}/>
        <GeneralLabel
            label="Uphill (meters)"
            value={Math.round(uphill_meters)}/>
        <GeneralLabel
            label="Downhill (meters)"
            value={Math.round(downhill_meters)}/>
        </>
    );
}

function SwitchOrgDest({ reverseOrgDest }: { reverseOrgDest: () => void }) {
    return <Button className="active:bg-secondary-dark" variant="secondary" onClick={reverseOrgDest}>Reverse directions</Button>
}
export interface SettingsProps {
    setAvoidHills: (value: number, commit: boolean) => void
    setPreferProtectedLanes: (value: number, commit: boolean) => void
    reverseOrgDest: () => void
    children?: React.ReactNode
}

export interface Energy {
    calories: number
    uphill_meters: number
    downhill_meters: number
    total_meters: number
}

export interface RouteInformation {
    energy: Energy
}

function Settings_({
                       setAvoidHills,
                       setPreferProtectedLanes,
                       energy,
                       reverseOrgDest,
                       children,
                       onClose // Receive onClose method
                   }: SettingsProps & RouteInformation & { onClose?: () => void }) {
    const desktopClasses = " sm:w-80 sm:top-0 sm:mt-8"
    return (
        <div className={"w-full sm:w-80 absolute bottom-20 right-0 z-10 pt-6" + desktopClasses}>
            <Card className="relative p-6 flex flex-col gap-5 m-5">
                {onClose && <button
                    onClick={onClose}
                    className="absolute top-0 right-0 m-2 text-gray-400 hover:text-gray-500"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
                }
                <MySlider
                    startValue={0.5}
                    onChange={setAvoidHills}
                    label={"Avoid steep hills"}
                    hoverDescription={
                        "Increase to prioritize avoiding steep hills (routes will have gradual slopes)"
                    }
                />
                <MySlider
                    startValue={0.5}
                    onChange={setPreferProtectedLanes}
                    label={"Prefer bike lanes"}
                    hoverDescription={"Increase to prioritize routes that use bike lanes."}
                />
                <SwitchOrgDest reverseOrgDest={reverseOrgDest}/>
                <CaloriesCounter energy={energy} />
                {children}
            </Card>
        </div>
    );
}

export const Settings = React.memo(Settings_);
