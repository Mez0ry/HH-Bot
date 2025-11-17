use thirtyfour::prelude::*;
use crate::selector_manager::SelectorManager;

#[derive(Clone, Debug)]
pub struct Vacancy{
    title : Option<String>,
    respond_button : Option<WebElement>,
    button_href : String,
    vacancy_element : WebElement
}

impl std::fmt::Display for Vacancy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Vacancy").field("title", &self.title).field("respond_button", &self.respond_button).field("button_href", &self.button_href).field("vacancy_element", &self.vacancy_element).finish()
    }
}

impl PartialEq for Vacancy  {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.button_href == other.button_href
    }
}
unsafe impl Send for Vacancy {
    
}

impl Vacancy {
    pub fn new(vacancy : WebElement) -> Self {
        Self {title : None , respond_button : None, button_href : String::new(), vacancy_element: vacancy}
    }
    
    pub async fn update_vacancy_fields(&mut self){
        let button_res = self.vacancy_element.find(SelectorManager::find_selector("vacancy_respond".to_string()).await.get_by()).await;
        
        match button_res{
            Ok(button_element)=>{ 
                self.respond_button = Some(button_element);
            },
            Err(err)=>{println!("button wasnt found, error: {}", err);}
        }

        match self.get_href().await {
            Some(href)=>{
                self.button_href = href;
            },
            None=>{}
        };

        let vacancy_title_selector = "[data-qa=\"serp-item__title-text\"]";
                
        match self.vacancy_element.find(By::Css(vacancy_title_selector)).await{
            Ok(vacancy_title_element)=>{
                let title_text= vacancy_title_element.text().await;
                match title_text {
                    Ok(actual_title)=>{self.title = Some(actual_title);},
                    Err(err)=>{println!("For vacancy title wasn't found, error: {}", err);}
                }
            },
            Err(err)=>{
                dbg!("parent_element wasnt found: {}", err);
            }
        }
    }

    pub async fn click_respond(&self) -> bool {
        if self.respond_button.is_some(){
            let button = self.respond_button.as_ref().unwrap();
            let _ = button.wait_until().clickable().await;
            let _ = button.wait_until().enabled().await;
            let _ = button.wait_until().displayed().await;

            let click_result = button.click().await;

            match click_result {
                Ok(_) => {
                    println!("Respond button clicked");
                    return true;
                },
                Err(e) => {
                    eprintln!("Failed to click button: {:?}", e);
                    return false;
                },
            };
        }

        false
    }

    pub async fn get_title(&self) -> String{
        match &self.title {
            Some(title_text)=>{
                return title_text.clone()
            },
            None => {
                return String::new();
            }
        }
    }

    pub async fn get_vacancy(&self) -> &WebElement{
        &self.vacancy_element
    }

    pub async fn get_button(&self) -> Option<&WebElement>{
            match &self.respond_button {
            Some(element)=>{
                return Some(element)
            },
            None => {
                None
            }
        }
    }

    pub async fn get_href(&self) -> Option<String>{
        let button = self.get_button().await;
        
        if button.is_some() {
            
            let href = button.unwrap().attr("href").await;

            match href {
                Ok(element)=>{
                    match element{
                        Some(href) => return Some(href),
                        None=>{return None;}
                    }
                },
                Err(_)=>{
                    println!("Failed to get href");
                    return None;
                }
            }
        }

        return None;
    }
}