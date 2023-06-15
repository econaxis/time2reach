// export const baseUrl: string = "http://localhost:3030"
// const apiUrl: string = "https://vx18yjnxac.execute-api.us-east-1.amazonaws.com/dev"
const LOCAL_API = true

const apiUrl: string = "https://d12zadp3znyab3.cloudfront.net"
// export const baseUrl: string = apiUrl
// export const mvtUrl: string = apiUrl

export const baseUrl: string = LOCAL_API ? "http://localhost:3030" : apiUrl
export const mvtUrl: string = apiUrl
