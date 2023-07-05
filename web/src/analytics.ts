import { LOCAL_API } from "./dev-api";

declare function sa_event(eventName: string, metadata?: Record<string, any>);

export default function track (name: string, properties: Record<string, any>) {
    properties.LOCAL = LOCAL_API
    sa_event(name, properties)
}