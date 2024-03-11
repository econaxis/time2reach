import { type ReactNode, useEffect, useRef, useState } from "react";
import { CiSettings } from "react-icons/ci";
import { Button } from "@/components/ui/button";
import { Settings } from "@/Settings";

// TODO clean this up
type SettingsProps = Parameters<typeof Settings>;
type OmittedSettingsProps = Omit<SettingsProps[0], 'onClose'>;

interface SettingsToggleProps extends OmittedSettingsProps {
    children?: ReactNode
}

function mediaQuery(query: string) {
    return window.matchMedia(query).matches;
}

const DESKTOP = mediaQuery("(min-width: 768px)"); // Example breakpoint for desktop
export function SettingsToggle({ children, ...rest }: SettingsToggleProps) {
    const [isSettingsVisible, setIsSettingsVisible] = useState(false);
    const settingsRef = useRef<HTMLDivElement>(null);

    const toggleSettings = () => {
        setIsSettingsVisible(x => !x);
    };

    const closeSettings = () => {
        setIsSettingsVisible(false);
    };

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (settingsRef.current && !settingsRef.current.contains(event.target as Node)) {
                closeSettings();
            }
        }

        document.addEventListener("mousedown", handleClickOutside);
        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, []);

    if (DESKTOP) {
        // For desktop, always show settings without toggle functionality
        return <Settings {...rest} onClose={undefined}> {children} </Settings>;
    } else {
        // For non-desktop, use the toggle functionality
        return (
            <>
                {!isSettingsVisible && (
                    <Button
                        className="fixed bottom-16 inset-x-0 w-max mx-auto"
                        variant="secondary"
                        onClick={toggleSettings}
                    >
                        <CiSettings size={25} className="mr-1" />
                        <span>Route Options</span>
                    </Button>
                )}
                {isSettingsVisible && (
                    <div ref={settingsRef}>
                        <Settings {...rest} onClose={closeSettings}> {children} </Settings>
                    </div>
                )}
            </>
        );
    }
}

export default SettingsToggle;
