import { useState, useRef, useEffect, type ReactNode } from "react";
import { CiSettings } from "react-icons/ci";
import { Button } from "@/components/ui/button";
import { Settings } from "@/Settings";

// TODO clean this up
type SettingsProps = Parameters<typeof Settings>;
type OmittedSettingsProps = Omit<SettingsProps[0], 'onClose'>;

interface SettingsToggleProps extends OmittedSettingsProps {
    children?: ReactNode
}

export function SettingsToggle({ children, ...rest }: SettingsToggleProps) {
    const [isSettingsVisible, setIsSettingsVisible] = useState(false);
    const settingsRef = useRef<HTMLDivElement>(null); // Typing the ref

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
    }, []); // Dependency array corrected to include the necessary dependencies

    return (
        <>
            {!isSettingsVisible && (
                <Button
                    className="fixed bottom-20 inset-x-0 w-max mx-auto"
                    variant="secondary"
                    onClick={toggleSettings}
                >
                    <CiSettings size={25} className="mr-1" />
                    <span>Route Options</span>
                </Button>
            )}
            {isSettingsVisible && (
                <div ref={settingsRef}>
                    <Settings {...rest} onClose={closeSettings} />
                </div>
            )}
        </>
    );
}

export default SettingsToggle;
