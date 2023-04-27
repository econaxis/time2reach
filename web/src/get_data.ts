import { type TimeColorMapper } from './colors'

export async function getDetails (data: TimeColorMapper, location: object) {
  const body = {
    request_id: data.request_id,
    latlng: location
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
