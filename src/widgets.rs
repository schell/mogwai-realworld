use mogwai::prelude::*;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

pub trait InputOrArea: IsDomNode {
    fn value(&self) -> String;

    fn set_value(&self, value: &str);

    fn view(
        text_input: &TextInput<Self>,
        tx: &Transmitter<TextInputIn<Self>>,
        rx: &Receiver<TextInputOut>,
    ) -> ViewBuilder<Self>;
}

pub struct TextInput<T> {
    o_input: Option<T>,
    pub value: String,
    pub placeholder: String,
}

impl<T> TextInput<T> {
    pub fn new(value: &str, placeholder: &str) -> Self {
        TextInput {
            o_input: None,
            value: value.to_string(),
            placeholder: placeholder.to_string(),
        }
    }
}

impl InputOrArea for HtmlInputElement {
    fn value(&self) -> String {
        HtmlInputElement::value(self)
    }

    fn set_value(&self, value: &str) {
        HtmlInputElement::set_value(self, value);
    }

    fn view(
        text_input: &TextInput<Self>,
        tx: &Transmitter<TextInputIn<Self>>,
        _rx: &Receiver<TextInputOut>,
    ) -> ViewBuilder<Self> {
        builder! {
            <input
                cast:type = Self
                post:build = tx.contra_map(|input: &Self| TextInputIn::PostBuild(input.clone()))
                on:keyup = tx.contra_map(|_| TextInputIn::UpdateValue)
                class="form-control"
                type="text"
                placeholder=&text_input.placeholder />
        }
    }
}

impl InputOrArea for HtmlTextAreaElement {
    fn value(&self) -> String {
        HtmlTextAreaElement::value(self)
    }

    fn set_value(&self, value: &str) {
        HtmlTextAreaElement::set_value(self, value);
    }

    fn view(
        text_area: &TextInput<Self>,
        tx: &Transmitter<TextInputIn<Self>>,
        _rx: &Receiver<TextInputOut>,
    ) -> ViewBuilder<Self> {
        builder! {
            <textarea
                cast:type = Self
                post:build = tx.contra_map(|input: &Self| TextInputIn::PostBuild(input.clone()))
                on:keyup = tx.contra_map(|_| TextInputIn::UpdateValue)
                class="form-control form-control-lg"
                rows="8"
                placeholder=&text_area.placeholder >
            </textarea>
        }
    }
}

#[derive(Clone)]
pub enum TextInputIn<T> {
    PostBuild(T),
    SetValue(String),
    UpdateValue,
}

#[derive(Clone)]
pub enum TextInputOut {
    UpdatedValue(String),
}

impl<T> Component for TextInput<T>
where
    T: IsDomNode + AsRef<Node> + InputOrArea,
{
    type ModelMsg = TextInputIn<T>;
    type ViewMsg = TextInputOut;
    type DomNode = T;

    fn update(
        &mut self,
        msg: &Self::ModelMsg,
        tx_view: &Transmitter<Self::ViewMsg>,
        sub: &Subscriber<Self::ModelMsg>,
    ) {
        match msg {
            TextInputIn::PostBuild(input) => {
                input.set_value(&self.value);
                self.o_input = Some(input.clone());
            }
            TextInputIn::SetValue(value) => {
                if let Some(input) = self.o_input.as_ref() {
                    input.set_value(value);
                }
                sub.send_async(async { TextInputIn::UpdateValue });
            }
            TextInputIn::UpdateValue => {
                if let Some(input) = self.o_input.as_ref() {
                    let old_value = std::mem::replace(&mut self.value, input.value());
                    if old_value != self.value {
                        tx_view.send(&TextInputOut::UpdatedValue(self.value.clone()));
                    }
                }
            }
        }
    }

    fn view(
        &self,
        tx: &Transmitter<Self::ModelMsg>,
        rx: &Receiver<Self::ViewMsg>,
    ) -> ViewBuilder<Self::DomNode> {
        T::view(self, tx, rx)
    }
}
