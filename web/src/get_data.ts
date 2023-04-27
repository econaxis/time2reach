import { type TimeColorMapper } from './colors'
import { type LngLat } from 'mapbox-gl'

export async function getDetails (data: TimeColorMapper, location: LngLat) {
  const body = {
    request_id: data.request_id,
    latlng: {
      latitude: location.lat,
      longitude: location.lng
    }
  }
  const resp = await fetch(
    'http://localhost:3030/details/',
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
