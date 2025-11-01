// https://github.com/jongiddy/cf-dioxus

#[cfg(feature = "api")]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MultiplyRequest {
  pub factor1: i32,
  pub factor2: i32,
}

#[cfg(feature = "api")]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MultiplyResponse {
  pub product: i32,
}

#[cfg(feature = "api")]
pub async fn multiply(factor1: i32, factor2: i32) -> Result<i32, std::io::Error> {
  let location = ::web_sys::window().unwrap().location().origin().unwrap();
  let mut url = reqwest::Url::parse(&location).map_err(std::io::Error::other)?;
  url.set_path("api/multiply");
  let query = serde_urlencoded::to_string(MultiplyRequest { factor1, factor2 })
    .map_err(std::io::Error::other)?;
  url.set_query(Some(&query));
  let response = reqwest::get(url).await.map_err(std::io::Error::other)?;

  if !response.status().is_success() {
    return Err(std::io::Error::other(response.status().to_string()));
  }

  let multiplication = response
    .json::<MultiplyResponse>()
    .await
    .map_err(std::io::Error::other)?;

  Ok(multiplication.product)
}
