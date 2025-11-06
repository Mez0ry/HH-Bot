use std::{collections::HashSet, result};
use thirtyfour::{WebDriver, error::WebDriverError, prelude::ElementWaitable};

use crate::{element_action::ElementAction, selector_manager::SelectorManager, vacancy::Vacancy};
/**
 * @Considerations 1. Probably need to make use of chain of responsibility design pattern with some base class of handlers
 */

/**
 * @brief probably will require fields such as elements_on_page, elements to skip
 * @TODO
 */
pub struct Page<'a> {
    driver: &'a  WebDriver,
    processed_vacancies: HashSet<String>,
    target_url: String,

    vacancies_on_page : usize, //@TODO these vars must be assign based on actual page filters
    vacancies_to_skip : usize
}

#[derive(PartialEq)]
enum ProcessingInfo{
    RedirectSuccess,
    Processed
}

enum ProcessingError{
    ButtonNotFound,
    HrefAlreadyProcessed,
    LimitReached,
    RelocationPopupFailure,
    AcceptCookiesFailure,
    SubmitButtonClickFailure,
    ResponseLetterFormFailure,
    RedirectFailure
}

pub enum PageProcessState{
    GatheringVacancies
}

pub enum GatheringInfo{
    Gathered,
    EmptyAfterGathered,
    DriverError
}

impl<'a> Page<'a> {
    pub fn new(target_url : String, driver : &'a WebDriver) -> Self {
        Page {target_url : target_url, driver: driver, processed_vacancies: HashSet::new(), vacancies_on_page: 49, vacancies_to_skip: 0 }
    }

    pub async fn process_vacancy(&mut self, vacancy: &Vacancy) -> Result<ProcessingInfo, ProcessingError> {
        println!("Processing vacancy: title: {}", vacancy.get_title().await);

        let respond_button = vacancy.get_button().await;
        let href = vacancy.get_href().await;

        if respond_button.is_none() || href.is_none() {
            return Err(ProcessingError::ButtonNotFound);
        }

        let href = href.unwrap();

        if self.processed_vacancies.contains(&href) {
            return Err(ProcessingError::HrefAlreadyProcessed);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        if !vacancy.click_respond().await {
            self.processed_vacancies.insert(href.clone());
            return Err(ProcessingError::SubmitButtonClickFailure);
        }

        let limit_check = ElementAction::new(&self.driver, SelectorManager::find_selector("vacancy_limit_reached").await);
        if ElementAction::try_exists(&limit_check, 3).await {
            return Err(ProcessingError::LimitReached);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let relocation_popup = ElementAction::new(&self.driver, SelectorManager::find_selector("relocation_warning_confirm").await);
        if ElementAction::try_exists(&relocation_popup, 3).await {
            ElementAction::try_safe_click(&relocation_popup, 3).await;
        }

        let accept_cookies = ElementAction::new(&self.driver, SelectorManager::find_selector("accept_cookies").await);
        if ElementAction::try_exists(&accept_cookies, 4).await {
            ElementAction::try_safe_click(&accept_cookies, 3).await;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let submit_button = ElementAction::new(&self.driver, SelectorManager::find_selector("submit_button").await);
        if ElementAction::try_exists(&submit_button, 3).await {
            ElementAction::try_safe_click(&submit_button, 3).await;

            let response_letter_toggle = ElementAction::new(&self.driver, SelectorManager::find_selector("response_letter_toggle").await);
            if ElementAction::try_exists(&response_letter_toggle, 3).await {
                ElementAction::try_safe_click(&response_letter_toggle, 3).await;

                let response_letter_form_input = ElementAction::new(&self.driver, SelectorManager::find_selector("response_letter_form_input").await);
                if ElementAction::try_exists(&response_letter_form_input, 3).await{
                    ElementAction::try_safe_click(&response_letter_form_input, 3).await;
                    response_letter_form_input.send_keys("test".to_string()).await;

                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    let submit_button = ElementAction::new(&self.driver, SelectorManager::find_selector("submit_button").await);
                    if ElementAction::try_exists(&submit_button, 3).await{
                        ElementAction::try_safe_click(&submit_button, 3).await;
                    }
                }
            }
        }

        self.processed_vacancies.insert(href.clone());

        let current_url = self.driver.current_url().await;

        if current_url.is_ok().to_string() != self.target_url.clone() {
            if ElementAction::try_exists(&limit_check, 3).await {
                return Err(ProcessingError::RedirectFailure);
            }

            let redirect_res = self.driver.goto(&self.target_url).await;
            match redirect_res{
                Ok(_)=>{
                    return Ok(ProcessingInfo::RedirectSuccess);
                },
                Err(err)=>{
                    println!("Error: {}, failed to redirect on page: {}", err, &self.target_url);
                }
            }
        }

        Ok(ProcessingInfo::Processed)
    }

    pub async fn gather_vacancies(&mut self, vacancies_out : &mut Vec<Vacancy>) -> GatheringInfo{
        if !vacancies_out.is_empty(){
            vacancies_out.clear();
        }

        let all_vacancies = self.driver.find_all(SelectorManager::find_selector("vacancy").await.get_by()).await;
        match all_vacancies {
            Ok(vacancies)=>{
                self.vacancies_to_skip = self.processed_vacancies.len() % self.vacancies_on_page;

                for vacancy_element in vacancies.iter().skip(self.vacancies_to_skip){
                    let mut cloned_vacancy = Vacancy::new(vacancy_element.clone());
                    let _ = vacancy_element.wait_until().clickable().await;

                    cloned_vacancy.update_vacancy_fields().await;

                    let respond_button = cloned_vacancy.get_button().await;
                    let title = cloned_vacancy.get_title().await;
                    let href = cloned_vacancy.get_href().await;

                    if respond_button.is_none() || title.is_empty() || href.is_none() || href.is_some_and(|actual_href| self.processed_vacancies.contains(&actual_href)){
                        eprintln!("Skipping vacancy with no button/title/href or already responded on it");
                        continue;
                    }

                    vacancies_out.push(cloned_vacancy);
                }
                
                if vacancies_out.is_empty(){
                    return GatheringInfo::EmptyAfterGathered;
                }

                return GatheringInfo::Gathered;
            },
            Err(err) => {
                println!("Failed to gather vacancies, Error: {}", err);
                return GatheringInfo::DriverError;
            }
        }

        unreachable!("by some unknown reason failed to gather vacancies, and its reached 'unreachable' state");
    }

    pub async fn process_page(& mut self) -> PageProcessState{
        let state = PageProcessState::GatheringVacancies;
        
        let mut vacancies : Vec<Vacancy> = vec![];

        match self.gather_vacancies(&mut vacancies).await {
            GatheringInfo::Gathered => {
                println!("Vacancies successfully gathered");
            },
            GatheringInfo::EmptyAfterGathered => {
                //retry
            },
            GatheringInfo::DriverError => return state,
        }

        let mut processing_page : bool = true;

        while processing_page{

            for vacancy in &vacancies {
                match self.process_vacancy(vacancy).await{
                    Ok(info) =>{
                        match info {
                            ProcessingInfo::RedirectSuccess => break,
                            ProcessingInfo::Processed => {},
                        }
                    },
                    Err(error) => {
                        match error{
                            ProcessingError::ButtonNotFound => {},
                            ProcessingError::HrefAlreadyProcessed => {},
                            ProcessingError::LimitReached => processing_page = false,
                            ProcessingError::RelocationPopupFailure => {},
                            ProcessingError::AcceptCookiesFailure => {},
                            ProcessingError::SubmitButtonClickFailure => {},
                            ProcessingError::ResponseLetterFormFailure => {},
                            ProcessingError::RedirectFailure => {},
                        }
                    },
                }

            }

            match self.gather_vacancies(&mut vacancies).await {
                GatheringInfo::Gathered => {
                    continue;
                },
                GatheringInfo::EmptyAfterGathered => {
                 self.move_to_next_page_if_any(&mut vacancies).await; // if failed goto to next search query
                },
                GatheringInfo::DriverError => return state,
            }
        }

        return state;
    }

    pub async fn move_to_next_page_if_any(&mut self, vacancies_out: &mut Vec<Vacancy>){
      
        let page_next = ElementAction::new(&self.driver, SelectorManager::find_selector("next_page").await);
        if ElementAction::try_exists(&page_next, 3).await{
            ElementAction::try_safe_click(&page_next,3).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            let url  = self.driver.current_url().await;
            if url.is_ok(){
                let url = url.unwrap();
                self.vacancies_to_skip = self.processed_vacancies.len() % self.vacancies_on_page;
                
                self.target_url = url.clone().to_string();
                let _ = self.driver.goto(&self.target_url).await;

                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                self.gather_vacancies(vacancies_out).await;
                self.processed_vacancies.clear();

                println!("moving to page: target_url {}", &self.target_url);
            }
        }
    }
}