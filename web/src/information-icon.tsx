// @ts-ignore
import { ReactComponent as InformationIcon1 } from "./information.svg";

export function InformationIcon(props: { onClick: () => void }) {
    const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);

    let background = "inherit";
    let margin = "1.2rem";
    if (isMobile) {
        background = "rgb(32, 32, 32)";
        margin = "0.1rem";
    }
    return (
        <InformationIcon1
            onClick={props.onClick}
            className="absolute bottom-0 right-0 z-50"
            style={{
                margin,
                backgroundColor: background,
                width: "40px",
                height: "40px",
            }}
        />
    );
}
