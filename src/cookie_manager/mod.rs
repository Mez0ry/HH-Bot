pub mod cookie_manager {

    use std::{fs::File, io::Write};
    use thirtyfour::Cookie;
    use thirtyfour::error::WebDriverErrorInfo;
    use thirtyfour::prelude::*;

    pub struct CookieManager {
        
    }

    impl CookieManager {
        pub async fn load_cookies(path: &String) -> Result<Option<Vec<Cookie>>, WebDriverError> {
            let cookies = CookieManager::parse_cookies(path).await;
            match cookies {
                Ok(ref _data) => println!("Cookies successfully parsed"),
                Err(ref err) => println!("error is:  {}", err),
            }

            return cookies;
        }

        pub async fn save_cookies(
            path: String,
            cookies: Vec<Cookie>,
        ) -> Result<(), WebDriverError> {
            if cookies.is_empty() {
                return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                    "invalid argument, cookies are empty".to_string(),
                )));
            }

            let json_data = serde_json::to_string_pretty(&cookies)?;
            let mut file = File::create(&path)?;
            file.write_all(json_data.as_bytes())?;
            Ok(())
        }

        pub async fn parse_cookies(path: &String) -> Result<Option<Vec<Cookie>>, WebDriverError> {
            let str_as_path = std::path::Path::new(&path);
            if !str_as_path.exists() || str_as_path.is_dir() {
                return Ok(None);
            }

            let content = std::fs::read_to_string(&str_as_path)?;
            let parsed_cookies: Vec<Cookie> = serde_json::from_str(&content)?;

            Ok(Some(parsed_cookies))
        }
    }
}
