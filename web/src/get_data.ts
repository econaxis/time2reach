import { type TimeColorMapper } from './colors'
import { type LngLat } from 'mapbox-gl'
import { baseUrl } from "./dev-api"

export async function getDetails (data: TimeColorMapper, location: LngLat) {
  const body = {
    request_id: data.request_id,
    latlng: {
      latitude: location.lat,
      longitude: location.lng
    }
  }
  const resp = await fetch(
    `${baseUrl}/details/`,
    {
      method: 'POST',
      mode: 'cors',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(body)
    }
  )

  return await resp.json()
}
