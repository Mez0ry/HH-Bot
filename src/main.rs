use std::{collections::HashSet};
use thirtyfour::prelude::*;

pub mod cookie_manager;
pub mod vacancy;

use crate::{cookie_manager::cookie_manager::CookieManager, vacancy::vacancy::Vacancy};

type ThirtyFourError = thirtyfour::error::WebDriverError; // Исправили тип ошибки

#[tokio::main]
async fn main() -> Result<(), ThirtyFourError> {
    let mut caps = DesiredCapabilities::chrome();

    caps.add_arg("--headless")?;
    
    caps.add_arg("--enable-debugger-agent")?;
    caps.add_arg("--remote-debugging-port=9222")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VoiceInteraction")?;
    caps.add_arg("--disable-speech-api")?;
    caps.add_arg("--disable-background-networking")?;
    caps.add_arg("--disable-background-timer-throttling")?;
    caps.add_arg("--disable-renderer-backgrounding")?;
    caps.add_arg("--disable-backgrounding-occluded-windows")?;
    caps.add_arg("--disable-client-side-phishing-detection")?;
    caps.add_arg("--disable-sync")?;
    caps.add_arg("--disable-translate")?;
    caps.add_arg("--disable-ipc-flooding-protection")?;
    caps.add_arg("--log-level=3")?;
    caps.add_arg("--enable-unsafe-swiftshader")?;

    let driver = WebDriver::new("http://localhost:49983", caps).await?;

    let target_url = "https://hh.ru/search/vacancy?text=%D0%9F%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%81%D1%82+C%2B%2B&salary=&ored_clusters=true&enable_snippets=true&hhtmFrom=vacancy_search_list&hhtmFromLabel=vacancy_search_line";

    driver.goto(target_url).await?;

    //let cookie_manager = cookie_manager::cookie_manager::CookieManager::new("../resources/Cookies/cookies.json".to_string());
    let cookie_json_path: String = String::from("./resources/cookies.json");

    if let Some(cookies) = CookieManager::load_cookies(&cookie_json_path).await?{
        for cookie in cookies {
            driver.add_cookie(cookie).await?;
        }
    }

    driver.refresh().await?;
    let vacancy_selector = "[class=\"vacancy-card--n77Dj8TY8VIUF0yM font-inter\"]";
    let all_vacancies = driver.find_all(By::Css(vacancy_selector)).await?;
    
    let mut vacancies_vec : Vec<Vacancy> = vec![];
    
    for vacancy_element in all_vacancies{
        let mut vacancy = Vacancy::new(vacancy_element);
        vacancy.update_vacancy_fields(&driver).await;

        vacancies_vec.push(vacancy);
    }

    // let respond_button_selector = "[data-qa=\"vacancy-serp__vacancy_response\"]";
    // let mut respond_buttons = driver.find_all(By::Css(respond_button_selector)).await?;
    
    // if respond_buttons.is_empty(){
    //     //handle
    // }

    // respond_buttons.iter().next_back().unwrap().wait_until().clickable().await?;

    let mut responded_buttons_set: HashSet<String> = std::collections::HashSet::new();

    loop{

        for vacancy in &vacancies_vec{
            let cloned_vacancy: &Vacancy = &vacancy.clone();

            let button_href = cloned_vacancy.get_button().await.attr("href").await?;

            if button_href.clone().is_some_and(|href| responded_buttons_set.contains(&href)){
                println!("already responded on this vacancy {}", button_href.as_ref().unwrap());
                continue;    
            }

            cloned_vacancy.click_respond().await;

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let relocation_warning_popup = match driver.find(By::Css("[data-qa=\"relocation-warning-confirm\"]")).await {
                Ok(el) => Some(el),
                Err(_) => {
                    // let href_copy = button_href.clone();
                    eprintln!("Failed to locate relocation warning popup:");
                    None
                },
            };

            if let Some(popup) = relocation_warning_popup {
                // Работаем с найденным элементом p
                let is_displayed = match popup.is_displayed().await {
                    Ok(displayed) => displayed,
                    Err(_) => {
                        eprintln!("Failed to determine if relocation warning popup is displayed");
                        false
                    },
                };
        
                let is_clickable = match popup.is_clickable().await {
                    Ok(clickable) => clickable,
                    Err(_) => {
                        eprintln!("Failed to determine if relocation warning popup is clickable");
                        false
                    },
                };
        
                if is_displayed && is_clickable {
                    match popup.click().await {
                        Ok(_) => eprintln!("Relocation warning popup clicked successfully"),
                        Err(err) => eprintln!("Failed to click relocation warning popup: {}", err),
                    };
                }
            }

            println!("pre condition of preparing to submit button ");
            let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
            let submit_button = driver.find(By::Css(submit_button_selector)).await;

            match submit_button{
                Ok(submit_button_el) =>{
                    submit_button_el.wait_until().displayed().await?;

                    let response_letter_toggle = "[data-qa=\"vacancy-response-letter-toggle\"]";
                    let response_letter_toggle_button = driver.find(By::Css(response_letter_toggle)).await;

                    match response_letter_toggle_button {
                        Ok(response_letter_toggle_button_el)=>{
                            match response_letter_toggle_button_el.click().await{
                                Ok(_)=>{
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                },
                                Err(_)=>{
                                    println!("couldnt be pressed");
                                }
                            }
                        },
                        Err(err)=>{
                            println!("Didnt found response letter toogle button, error: {}", err);
                        }
                    };

                    let response_popup_form_selector = "[data-qa=\"vacancy-response-popup-form-letter-input\"]";
                    let letter_input_form = driver.find(By::Css(response_popup_form_selector)).await;

                    match letter_input_form{
                        Ok(letter_input_form_el) => {

                            letter_input_form_el.click().await?;
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            letter_input_form_el.send_keys("test").await?;

                            let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
                            let submit_button = driver.find(By::Css(submit_button_selector)).await?;

                            submit_button.wait_until().displayed().await?;
                            submit_button.wait_until().enabled().await?;
                            submit_button.wait_until().clickable().await?;

                            match submit_button.click().await {
                                Ok(_)=>{
                                    println!("everything ok");
                                },
                                Err(err)=>{
                                    println!("Submit button can't be pressed: {}", err);
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            println!("44");
                        },
                        Err(err)=> {
                            println!("Didnt found response letter toogle button, error: {}", err);
                        }
                    };

                    println!("end");
                    match button_href{
                        Some(href) => {
                            responded_buttons_set.insert(href.to_string());
                        },
                        None => {}
                    }

                },
                Err(_)=>{
                    println!("submit button ERR");
                    continue;
                }
            }

            let current_url = driver.current_url().await?;

            if current_url.as_str() != target_url {
                driver.goto(target_url).await?;
                break;
            } else {
                println!("Already on desired page, skipping navigation");
            }

            println!("end_button");
        }

        
        // for button in respond_buttons {
            
        //     println!("start");

        //     let button_href = button.attr("href").await?;

        //     let button_parent = button.find(By::XPath("ancestor::*[4]")).await;
        //     println!("after button_parent");

        //     #[allow(unused)] let mut vacancy_title : String;

        //     match button_parent {
        //         Ok(parent_element)=>{
        //             let vacancy_title_selector = "[data-qa=\"serp-item__title-text\"]";
                    
        //             match parent_element.find(By::Css(vacancy_title_selector)).await{
        //                 Ok(vacancy_title_element)=>{
        //                     vacancy_title = vacancy_title_element.text().await?;
        //                     println!("vacancy title: {}", vacancy_title);
        //                 },
        //                 Err(err)=>{
        //                     println!("parent_element wasnt found: {}", err);
        //                 }
        //             }

        //         },
        //         Err(err)=>{
        //             println!("Cant handle button parent: {}", err);
        //         }
        //     };

        //     println!("after match button_parent");
            
            // if button_href.clone().is_some_and(|href| responded_buttons_set.contains(&href)){
            //     println!("already responded on this vacancy {}", button_href.unwrap());
            //     continue;    
            // }

            // match button.click().await {
            //     Ok(_) => {
            //         let _ = match button_href {
            //             Some(_ ) =>  { },
            //             None => {},
            //         };
            //     },
            //     Err(e) => {eprintln!("Failed to click button: {}", e); },
            // };

            // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // let relocation_warning_popup = match driver.find(By::Css("[data-qa=\"relocation-warning-confirm\"]")).await {
            //     Ok(el) => Some(el),
            //     Err(_) => {
            //         // let href_copy = button_href.clone();
            //         eprintln!("Failed to locate relocation warning popup:");
            //         None
            //     },
            // };

            // if let Some(popup) = relocation_warning_popup {
            //     // Работаем с найденным элементом p
            //     let is_displayed = match popup.is_displayed().await {
            //         Ok(displayed) => displayed,
            //         Err(_) => {
            //             eprintln!("Failed to determine if relocation warning popup is displayed");
            //             false
            //         },
            //     };
        
            //     let is_clickable = match popup.is_clickable().await {
            //         Ok(clickable) => clickable,
            //         Err(_) => {
            //             eprintln!("Failed to determine if relocation warning popup is clickable");
            //             false
            //         },
            //     };
        
            //     if is_displayed && is_clickable {
            //         match popup.click().await {
            //             Ok(_) => eprintln!("Relocation warning popup clicked successfully"),
            //             Err(err) => eprintln!("Failed to click relocation warning popup: {}", err),
            //         };
            //     }
            // }

            // println!("pre condition of preparing to submit button ");
            // let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
            // let submit_button = driver.find(By::Css(submit_button_selector)).await;

            // match submit_button{
            //     Ok(submit_button_el) =>{
            //         submit_button_el.wait_until().displayed().await?;

            //         let response_letter_toggle = "[data-qa=\"vacancy-response-letter-toggle\"]";
            //         let response_letter_toggle_button = driver.find(By::Css(response_letter_toggle)).await;

            //         match response_letter_toggle_button {
            //             Ok(response_letter_toggle_button_el)=>{
            //                 match response_letter_toggle_button_el.click().await{
            //                     Ok(_)=>{
            //                         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            //                     },
            //                     Err(_)=>{
            //                         println!("couldnt be pressed");
            //                     }
            //                 }
            //             },
            //             Err(err)=>{
            //                 println!("Didnt found response letter toogle button, error: {}", err);
            //             }
            //         };

            //         let response_popup_form_selector = "[data-qa=\"vacancy-response-popup-form-letter-input\"]";
            //         let letter_input_form = driver.find(By::Css(response_popup_form_selector)).await;

            //         match letter_input_form{
            //             Ok(letter_input_form_el) => {

            //                 letter_input_form_el.click().await?;
            //                 tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            //                 letter_input_form_el.send_keys("test").await?;

            //                 let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
            //                 let submit_button = driver.find(By::Css(submit_button_selector)).await?;

            //                 submit_button.wait_until().displayed().await?;
            //                 submit_button.wait_until().enabled().await?;
            //                 submit_button.wait_until().clickable().await?;

            //                 match submit_button.click().await {
            //                     Ok(_)=>{
            //                         println!("everything ok");
            //                     },
            //                     Err(err)=>{
            //                         println!("Submit button can't be pressed: {}", err);
            //                     }
            //                 }

            //                 tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            //                 println!("44");
            //             },
            //             Err(err)=> {
            //                 println!("Didnt found response letter toogle button, error: {}", err);
            //             }
            //         };

            //         println!("end");
            //         match button_href{
            //             Some(href) => {
            //                 responded_buttons_set.insert(href.to_string());
            //             },
            //             None => {}
            //         }

            //     },
            //     Err(_)=>{
            //         println!("submit button ERR");
            //         continue;
            //     }
            // }

            // let current_url = driver.current_url().await?;

            // if current_url.as_str() != target_url {
            //     driver.goto(target_url).await?;
            //     break;
            // } else {
            //     println!("Already on desired page, skipping navigation");
            // }

            // println!("end_button");
        // }

        // println!("out of buttons loop");
        // respond_buttons = driver.find_all(By::Css(respond_button_selector)).await?;
                
        // if respond_buttons.is_empty(){
        //     //handle
        // }

        // respond_buttons.iter().next_back().unwrap().wait_until().clickable().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }// !loop

    CookieManager::save_cookies(cookie_json_path, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    driver.quit().await?;

    Ok(())
}