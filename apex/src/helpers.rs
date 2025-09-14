use crate::signal::Signal;
use std::collections::HashMap;
use std::rc::Rc;

/// Generic event handler used in component props
pub type EventHandler<E> = Rc<dyn Fn(E)>;

/// Creates a no-op event handler for any event type
pub fn noop_event<E>() -> EventHandler<E> {
    Rc::new(|_event: E| {})
}

#[derive(Clone, Debug, Default)]
pub struct HtmlAttributes {
    map: HashMap<String, String>,
}

pub trait IntoHtmlAttributes {
    fn into_attrs(self) -> HtmlAttributes;
}

impl IntoHtmlAttributes for HtmlAttributes {
    fn into_attrs(self) -> HtmlAttributes {
        self
    }
}

impl IntoHtmlAttributes for String {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set("class", self);
        a
    }
}

impl IntoHtmlAttributes for &str {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set("class", self);
        a
    }
}

impl IntoHtmlAttributes for (String, String) {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set(self.0, self.1);
        a
    }
}

impl IntoHtmlAttributes for (&str, &str) {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set(self.0, self.1);
        a
    }
}

impl IntoHtmlAttributes for (&str, String) {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set(self.0, self.1);
        a
    }
}

impl IntoHtmlAttributes for (String, &str) {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        a.set(self.0, self.1);
        a
    }
}

impl IntoHtmlAttributes for Vec<(String, String)> {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        for (k, v) in self.into_iter() {
            a.set(k, v);
        }
        a
    }
}

impl IntoHtmlAttributes for Vec<(&str, &str)> {
    fn into_attrs(self) -> HtmlAttributes {
        let mut a = HtmlAttributes::new();
        for (k, v) in self.into_iter() {
            a.set(k, v);
        }
        a
    }
}

impl<T: IntoHtmlAttributes> IntoHtmlAttributes for Option<T> {
    fn into_attrs(self) -> HtmlAttributes {
        match self {
            Some(t) => t.into_attrs(),
            None => HtmlAttributes::default(),
        }
    }
}

impl IntoHtmlAttributes for bool {
    fn into_attrs(self) -> HtmlAttributes {
        let mut attrs = HtmlAttributes::new();
        if self {
            attrs.set("data-value", "true");
        }
        attrs
    }
}

impl<T: Clone + ToString + 'static> IntoHtmlAttributes for Signal<T> {
    fn into_attrs(self) -> HtmlAttributes {
        let mut attrs = HtmlAttributes::new();
        attrs.set("data-signal", self.get().to_string());
        attrs
    }
}

pub fn into_html_attrs<T: IntoHtmlAttributes>(t: T) -> HtmlAttributes {
    t.into_attrs()
}

impl HtmlAttributes {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        if name == "class" {
            self.merge_class(&value);
        } else if name == "style" {
            self.merge_style(&value);
        } else {
            self.map.insert(name, value);
        }
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.set(name, value);
        self
    }

    pub fn merge(&mut self, other: HtmlAttributes) {
        for (k, v) in other.map.into_iter() {
            self.set(k, v);
        }
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.map.get(name)
    }

    pub fn class(&self) -> Option<&String> {
        self.map.get("class")
    }

    pub fn style(&self) -> Option<&String> {
        self.map.get("style")
    }

    pub fn iter_owned(self) -> impl Iterator<Item = (String, String)> {
        self.map.into_iter()
    }

    fn merge_class(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        match self.map.get_mut("class") {
            Some(existing) => {
                if !existing.is_empty() {
                    existing.push(' ');
                }
                existing.push_str(value);
            }
            None => {
                self.map.insert("class".to_owned(), value.to_owned());
            }
        }
    }

    fn merge_style(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        match self.map.get_mut("style") {
            Some(existing) => {
                if !existing.trim_end().ends_with(';') {
                    existing.push(';');
                }
                if !existing.is_empty() {
                    existing.push(' ');
                }
                existing.push_str(value);
            }
            None => {
                self.map.insert("style".to_owned(), value.to_owned());
            }
        }
    }
}
