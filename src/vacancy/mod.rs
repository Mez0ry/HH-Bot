pub mod vacancy {
    use thirtyfour::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Vacancy{
        title : Option<String>,
        respond_button : Option<WebElement>,
        vacancy_element : WebElement
    }
    
    impl Vacancy {
        pub fn new(vacancy : WebElement) -> Self {
            Self {title : None , respond_button : None, vacancy_element: vacancy}
        }

        pub async fn update_vacancy_fields(&mut self, driver : &WebDriver){
            let respond_button_selector = "[data-qa=\"vacancy-serp__vacancy_response\"][class*=\"magritte-button_stretched\"]";
            let button_res = self.vacancy_element.find(By::Css(respond_button_selector)).await;

            match button_res{
                Ok(button_element)=>{ 
                    self.respond_button = Some(button_element);
                    println!("button found");
                },
                Err(err)=>{println!("button wasnt found, error: {}", err);}
            }

            let vacancy_title_selector = "[data-qa=\"serp-item__title-text\"]";
                    
            match self.vacancy_element.find(By::Css(vacancy_title_selector)).await{
                Ok(vacancy_title_element)=>{
                    let title_text= vacancy_title_element.text().await;
                    match title_text {
                        Ok(actual_title)=>{self.title = Some(actual_title);},
                        Err(err)=>{println!("For vacancy title wasn't found, error: {}", err);}
                    }

       
                    println!("vacancy title: {}", self.title.clone().unwrap());
                },
                Err(err)=>{
                    println!("parent_element wasnt found: {}", err);
                }
            }
        }

        pub async fn click_respond(&self){
            
            let is_clickable = async |_element : &Vacancy| -> bool {
                match &self.respond_button {
                    Some(elem) => {
                        let success = elem.is_clickable().await;
                        Some(success);
                    },
                    None => {},
                }
                false
            };

            if self.respond_button.is_some(){
                let button = self.respond_button.as_ref().unwrap();

                if is_clickable(&self).await{

                }

                let click_result = button.click().await;

                match click_result {
                    Ok(_) => println!("Respond button clicked"),
                    Err(e) => eprintln!("Failed to click button: {:?}", e),
                };

            }
        }

        pub async fn get_title(&self) -> &String{
            match &self.title {
                Some(title_text)=>{
                    return title_text
                },
                None => panic!("title is None")
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

        pub async fn get_href(&self) -> String{
            let button = self.get_button().await;
            let href = button.unwrap().attr("href").await;

            match href {
                Ok(element)=>{
                    let actual_href = element.is_some().to_string();
                    return actual_href;
                },
                Err(_)=>{
                   println!("Failed to get href");
                   return String::new();
                }
            }
        }
    }
}