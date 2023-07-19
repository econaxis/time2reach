// export const baseUrl: string = "http://localhost:3030"
// const apiUrl: string = "https://vx18yjnxac.execute-api.us-east-1.amazonaws.com/dev"

export const IS_LOCAL = import.meta.env.MODE === "development";
export const LOCAL_API: boolean = import.meta.env.MODE === "development" && false;

// const apiUrl: string = "https://d12zadp3znyab3.cloudfront.net"
// const apiUrl: string =
// export const baseUrl: string = apiUrl
// export const mvtUrl: string = apiUrl

// export const baseUrl: string = LOCAL_API ? "http://localhost:3030" : apiUrl
export let baseUrl: string = LOCAL_API ? "http://127.0.0.1:3030" : "https://map.henryn.xyz/api";

if (IS_LOCAL && false) {
    baseUrl = "http://127.0.0.1:3030";
}

export const mvtUrl: string = LOCAL_API
    ? "http://127.0.0.1:3030/mvt"
    : "https://map.henryn.xyz/api/mvt";
// export const mvtUrl: string = LOCAL_API ? 'http://127.0.0.1:6767' : apiUrl

// @ts-expect-error window
window.sa_metadata = { local: LOCAL_API };
