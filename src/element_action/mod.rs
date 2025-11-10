use thirtyfour::{error::WebDriverErrorInfo, prelude::*};
use crate::{retry, selector::MySelector};

pub struct ElementAction<'a> {
    driver: &'a WebDriver,
    selector : MySelector
}

impl<'a> ElementAction<'a> {
    pub fn new(driver: &'a WebDriver, my_selector: MySelector ) -> Self {
        ElementAction { driver : driver, selector : my_selector}
    }

    pub async fn exists(&self) -> Result<bool, WebDriverError> {
        match self.find_element().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn is_displayed(&self) -> Result<bool, WebDriverError> {
        let element = self.find_element().await?;
        element.is_displayed().await
    }

    pub async fn is_clickable(&self) -> bool {
        let element = self.find_element().await;

        match element {
            Ok(element)=>{
                match element.is_clickable().await{
                    Ok(success)=> return success,
                    Err(_) => return false
                }
            },
            Err(err)=>{
                println!("Failed to find element, err: {}", err);
            }
        }
        false
    }
    
    pub async fn click(&self) -> Result<(), WebDriverError> {
        let element = self.find_element().await?;
        element.click().await
    }

    pub async fn safe_click(&self) -> Result<(), WebDriverError> {
        if self.exists().await? && self.is_displayed().await? && self.is_clickable().await {
            self.click().await?;
            Ok(())
        } else {
            Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("cant perform action on element which does not meet action requirements".to_string())))
        }
    }

    pub async fn send_keys(&self, keys : String) -> bool{
        let result_element = self.find_element().await;

        let element = result_element.unwrap();

        match element.send_keys(keys).await{
            Ok(())=>{
                true
            },
            Err(err)=>{
                println!("Failed to send keys, error: {}", err);
                false
            }
        }
    }

    pub async fn try_exists(action: &ElementAction<'_>, retries: u32) -> bool {
        return retry::retry_on_err(async |attempt : u32| {
            match action.exists().await {
                Ok(stream) => return stream,
                Err(_) => {
                    println!("element doesnt exists, attempt {}", attempt);
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    false
                }
            }
        }, retries).await;
    }

    pub async fn try_safe_click(action: &ElementAction<'_>, retries: u32) -> bool{
        return retry::retry_on_err(async |attempt : u32| {
            match action.safe_click().await {
                Ok(_) => return true,
                Err(_) => {
                    println!("safe_click attempt {}", attempt);
                    
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }

                    false
                }
            }
        }, retries).await;
    }

    async fn find_element(&self) -> WebDriverResult<WebElement>{
        self.driver.find(self.selector.clone().get_by()).await
    }

}