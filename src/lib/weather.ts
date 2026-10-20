// Current weather from Open-Meteo (free, no API key).

export type Weather = { temp: number; icon: string };

/** WMO weather code → emoji. */
function icon(code: number, day: boolean): string {
  if (code === 0) return day ? "☀️" : "🌙";
  if (code <= 2) return day ? "🌤️" : "☁️";
  if (code === 3) return "☁️";
  if (code <= 48) return "🌫️";
  if (code <= 67) return "🌧️";
  if (code <= 77) return "❄️";
  if (code <= 82) return "🌦️";
  return "⛈️";
}

export async function fetchWeather(latitude: number, longitude: number): Promise<Weather | null> {
  try {
    const q = new URLSearchParams({
      latitude: String(latitude),
      longitude: String(longitude),
      current: "temperature_2m,weather_code,is_day",
    });
    const res = await fetch(`https://api.open-meteo.com/v1/forecast?${q}`);
    if (!res.ok) return null;
    const { current } = await res.json();
    return { temp: Math.round(current.temperature_2m), icon: icon(current.weather_code, current.is_day === 1) };
  } catch {
    return null;
  }
}

export type Place = { name: string; latitude: number; longitude: number; detail: string };

/** City search for the settings window. */
export async function searchCity(name: string): Promise<Place[]> {
  const q = new URLSearchParams({ name, count: "6", language: "id" });
  const res = await fetch(`https://geocoding-api.open-meteo.com/v1/search?${q}`);
  if (!res.ok) return [];
  const { results = [] } = await res.json();
  return results.map((r: any) => ({
    name: r.name,
    latitude: r.latitude,
    longitude: r.longitude,
    detail: [r.admin1, r.country].filter(Boolean).join(", "),
  }));
}
