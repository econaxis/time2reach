// eslint-disable-next-line @typescript-eslint/naming-convention
declare function sa_event(eventName: string, metadata?: Record<string, any>);

export default function track(name: string, properties: Record<string, any>) {
    try {
        // @ts-expect-error window
        if (window.sa_event) {
            sa_event(name, properties);
        }
    } catch (e) {
        console.error("Error sending analytics", e);
    }
}
