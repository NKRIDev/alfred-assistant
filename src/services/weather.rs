use std::thread;
use serde_json::Value;

pub struct WeatherService;

impl WeatherService {

    /*
    Returns the weather for a city
     */
    pub fn get_weather_city(city: &str, date: Option<&str>) -> String {
        let city = city.to_string();
        let date = date.map(|d| d.to_string());

        let handle = thread::spawn(move || {
            /*
            City geolocation (retrieval of latitude and longitude)
             */
            let geo_api = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=10&language=fr&format=json",
                                  city);

            /*
            Call api to recover localisation
             */
            let geo_response = match reqwest::blocking::get(&geo_api) {
                Ok(resp) => match resp.text() {
                    Ok(body) => body,
                    Err(e) => return format!("Geolocalized reading error : {}", e),
                },
                Err(e) => return format!("Geolocalized request error : {}", e),
            };

            /*
            Transform string to json
             */
            let geo_json: Value = match serde_json::from_str(&geo_response) {
                Ok(json) => json,
                Err(e) => return format!("Parsing error to geolocalized : {}", e),
            };

            /*
            Recover results section in json
             */
            let results = match geo_json["results"].as_array() {
                Some(r) if !r.is_empty() => r,
                _ => return format!("City '{}' not found", city),
            };

            /*
            Recover the first result
             */
            let first = &results[0];
            let lat = first["latitude"].as_f64().unwrap_or(0.0);
            let lon = first["longitude"].as_f64().unwrap_or(0.0);
            let name = first["name"].as_str().unwrap_or(&city);
            let country = first["country"].as_str().unwrap_or("?");

            /*
            Weather API URL
             */
            let weather_api = match &date {
                Some(date) => format!(
                    "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,\
                    weather_code,relative_humidity_2m_max,wind_speed_10m_max,apparent_temperature_max&start_date={}&end_date={}&timezone=auto",
                    lat, lon, date, date
                ),
                None => format!(
                    "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,\
                    weather_code,relative_humidity_2m,wind_speed_10m,apparent_temperature",
                    lat, lon
                ),
            };

            let weather_body = match reqwest::blocking::get(&weather_api) {
                Ok(resp) => match resp.text() {
                    Ok(body) => body,
                    Err(e) => return format!("Error with weather : {}", e),
                },
                Err(e) => return format!("Request weather error : {}", e),
            };

            format!("City found : {}, {}\n{}, {}", name, country, weather_body, date.unwrap_or("?".to_string()))
        });

        match handle.join() {
            Ok(result) => result,
            Err(_) => String::from("Panic thread !"),
        }
    }
}