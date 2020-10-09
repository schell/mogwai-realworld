//! The login/register component.
//!
//! Receives model messages about changing routes and outputs view messages about changing
//! routes.
#![allow(unused_braces)]
use mogwai::prelude::*;
use web_sys::{HtmlInputElement, Location};

use crate::{
    api::{self, User, UserRegistration},
    route::Route,
    store,
};

/// The registration UI component.
pub struct Register {
    username_input: Option<HtmlInputElement>,
    email_input: Option<HtmlInputElement>,
    password_input: Option<HtmlInputElement>,
}

impl Default for Register {
    fn default() -> Self {
        Register {
            username_input: None,
            email_input: None,
            password_input: None,
        }
    }
}

impl Register {
    fn get_registration(&self) -> UserRegistration {
        let get_value = |o_input: &Option<HtmlInputElement>| -> Option<String> {
            o_input
                .as_ref()
                .map(|input| match input.value().as_str() {
                    "" => None,
                    s => Some(s.into()),
                })
                .flatten()
        };
        let username = get_value(&self.username_input);
        let email = get_value(&self.email_input);
        let password = get_value(&self.password_input);
        UserRegistration {
            username,
            email,
            password,
        }
    }
}

#[derive(Clone, Debug)]
pub enum In {
    UsernameInput(HtmlInputElement),
    EmailInput(HtmlInputElement),
    PasswordInput(HtmlInputElement),
    Submit,
    RegistrationSuccess { user: User },
    RegistrationFailure { errors: Vec<String> },
}

#[derive(Clone)]
pub enum Out {
    Error(Patch<View<HtmlElement>>),
}

impl Out {
    fn errors(&self) -> Option<Patch<View<HtmlElement>>> {
        match self {
            Out::Error(patch) => Some(patch.clone()),
        }
    }
}

impl Component for Register {
    type ModelMsg = In;
    type ViewMsg = Out;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &In, tx: &Transmitter<Out>, sub: &Subscriber<In>) {
        match msg {
            In::UsernameInput(input) => {
                self.username_input = Some(input.clone());
            }
            In::EmailInput(input) => {
                self.email_input = Some(input.clone());
            }
            In::PasswordInput(input) => {
                self.password_input = Some(input.clone());
            }
            In::Submit => {
                let registration = self.get_registration();
                sub.send_async(async {
                    match api::register_user(registration).await {
                        Ok(user) => In::RegistrationSuccess { user },
                        Err(err) => {
                            let errors = match err {
                                api::request::Error::ResponseErrors { errors } => errors
                                    .into_iter()
                                    .flat_map(|(name, descs)| -> Vec<String> {
                                        descs
                                            .into_iter()
                                            .map(|desc| format!("{} {}", name, desc))
                                            .collect()
                                    })
                                    .collect(),
                                _ => vec![format!("{}", err)],
                            };
                            In::RegistrationFailure { errors }
                        }
                    }
                });
            }
            In::RegistrationSuccess { user } => match store::write_user(user) {
                Ok(()) => {
                    let location: Location = mogwai::utils::window().location();
                    let _ = location.set_hash(&Route::Home.as_hash());
                }
                Err(err) => {
                    sub.send_async(async move {
                        In::RegistrationFailure {
                            errors: vec![format!("{}", err)],
                        }
                    });
                }
            },
            In::RegistrationFailure { errors } => {
                tx.send(&Out::Error(Patch::RemoveAll));
                for error in errors.iter() {
                    tx.send(&Out::Error(Patch::PushBack {
                        value: view! {
                            <li>{error}</li>
                        },
                    }));
                }
            }
        }
    }

    fn view(&self, tx: &Transmitter<In>, rx: &Receiver<Out>) -> ViewBuilder<HtmlElement> {
        builder! {
            <div class="auth-page">
                <div class="container page">
                    <div class="row">
                        <div class="col-md-6 offset-md-3 col-xs-12">
                            <h1 class="text-xs-center">"Sign up"</h1>
                            <p class="text-xs-center">
                                <a href="#/login">"Have an account?"</a>
                            </p>

                            <ul class="error-messages"
                                patch:children=rx.branch_filter_map(|msg| msg.errors())>
                            </ul>

                            <form>
                                <fieldset class="form-group">
                                    <input
                                        cast:type=web_sys::HtmlInputElement
                                        class="form-control form-control-lg"
                                        type="text"
                                        placeholder="Your Name"
                                        post:build=tx.contra_map(|el:&HtmlInputElement| {
                                            In::UsernameInput(el.clone())
                                        })
                                        />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input
                                        cast:type=web_sys::HtmlInputElement
                                        class="form-control form-control-lg"
                                        type="text"
                                        placeholder="Email"
                                        post:build=tx.contra_map(|el:&HtmlInputElement| {
                                            In::EmailInput(el.clone())
                                        })
                                        />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input
                                        cast:type=web_sys::HtmlInputElement
                                        class="form-control form-control-lg"
                                        type="password"
                                        placeholder="Password"
                                        post:build=tx.contra_map(|el:&HtmlInputElement| {
                                            In::PasswordInput(el.clone())
                                        })
                                        />
                                </fieldset>
                                <button
                                    class="btn btn-lg btn-primary pull-xs-right"
                                    on:click=tx.contra_map(|ev: &Event| {
                                        ev.prevent_default();
                                        In::Submit
                                    })>
                                    "Sign up"
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
