use scraper::element_ref::Select;
use scraper::{ElementRef, Html, Selector};

pub trait SelectorExtend {
    fn select<'a, 'b>(&'a self, selector: &'b Selector) -> Select<'a, 'b>;

    fn select_text(&self, selector: &str) -> Option<String> {
        let selector = Selector::parse(selector).unwrap();
        self.select(&selector)
            .next()?
            .text()
            .next()?
            .to_string()
            .into()
    }

    fn select_texts(&self, selector: &str) -> Vec<String> {
        let selector = Selector::parse(selector).unwrap();
        self.select(&selector)
            .filter_map(|e| e.text().next().map(|t| t.to_string()))
            .collect()
    }

    fn select_attr(&self, selector: &str, attr: &str) -> Option<String> {
        let selector = Selector::parse(selector).unwrap();
        self.select(&selector)
            .next()?
            .value()
            .attr(attr)?
            .to_string()
            .into()
    }

    fn select_attrs(&self, selector: &str, attr: &str) -> Vec<String> {
        let selector = Selector::parse(selector).unwrap();
        self.select(&selector)
            .filter_map(|e| e.value().attr(attr).map(|t| t.to_string()))
            .collect()
    }
}

impl SelectorExtend for Html {
    fn select<'a, 'b>(&'a self, selector: &'b Selector) -> Select<'a, 'b> {
        self.root_element().select(selector)
    }
}

impl<'a> SelectorExtend for ElementRef<'a> {
    fn select<'b, 'c>(&'b self, selector: &'c Selector) -> Select<'b, 'c> {
        self.select(selector)
    }
}
