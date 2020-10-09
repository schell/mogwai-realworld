use mogwai::prelude::*;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

use crate::{
    api::{self, User, UserUpdate},
    route::Route,
    store,
};

/// The settings UI component.
pub struct Settings {
    o_user: Option<User>,
    o_pic_input: Option<HtmlInputElement>,
    o_name_input: Option<HtmlInputElement>,
    o_bio_input: Option<HtmlTextAreaElement>,
    o_email_input: Option<HtmlInputElement>,
    o_password_input: Option<HtmlInputElement>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            o_user: store::read_user().ok(),
            o_pic_input: None,
            o_name_input: None,
            o_bio_input: None,
            o_email_input: None,
            o_password_input: None,
        }
    }
}

#[derive(Clone)]
pub enum In {
    ImageInput(HtmlInputElement),
    UsernameInput(HtmlInputElement),
    BioInput(HtmlTextAreaElement),
    EmailInput(HtmlInputElement),
    PasswordInput(HtmlInputElement),
    Submit,
    UpdateSuccess(User),
    UpdateFailure { errors: Vec<String> },
    Logout,
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

impl Component for Settings {
    type ModelMsg = In;
    type ViewMsg = Out;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &In, tx: &Transmitter<Out>, sub: &Subscriber<In>) {
        match msg {
            In::ImageInput(i) => {
                if let Some(url) = self.o_user.as_ref().map(|u| u.image.as_ref()).flatten() {
                    i.set_value(url);
                }
                self.o_pic_input = Some(i.clone());
            }
            In::UsernameInput(i) => {
                if let Some(username) = self.o_user.as_ref().map(|u| &u.username) {
                    i.set_value(username);
                }
                self.o_name_input = Some(i.clone());
            }
            In::BioInput(a) => {
                if let Some(bio) = self.o_user.as_ref().map(|u| u.bio.as_ref()).flatten() {
                    a.set_value(bio);
                }
                self.o_bio_input = Some(a.clone());
            }
            In::EmailInput(i) => {
                if let Some(email) = self.o_user.as_ref().map(|u| &u.email) {
                    i.set_value(email);
                }
                self.o_email_input = Some(i.clone());
            }
            In::PasswordInput(i) => {
                self.o_password_input = Some(i.clone());
            }
            In::Submit => {
                let email: Option<String> = self.o_email_input.as_ref().map(|i| i.value());
                let username: Option<String> = self.o_name_input.as_ref().map(|i| i.value());
                let bio: Option<String> = self.o_bio_input.as_ref().map(|i| i.value());
                let image: Option<String> = self.o_pic_input.as_ref().map(|i| i.value());
                let password: Option<String> = self
                    .o_password_input
                    .as_ref()
                    .map(|i| i.value())
                    .map(|s| if s.is_empty() { None } else { Some(s) })
                    .flatten();

                if let Some(user) = self.o_user.as_ref() {
                    let user_update = UserUpdate {
                        email,
                        username,
                        bio,
                        image,
                        password,
                    };
                    let token = user.token.clone();
                    sub.send_async(async move {
                        match api::update_user(user_update, &token).await {
                            Ok(user) => In::UpdateSuccess(user),
                            Err(err) => In::UpdateFailure {
                                errors: Vec::from(err),
                            },
                        }
                    })
                }
            }
            In::UpdateSuccess(user) => {
                self.o_user = Some(user.clone());

                let _ = store::write_user(user);
                let _ = mogwai::utils::window().location().set_hash(
                    Route::Profile {
                        username: user.username.clone(),
                        is_favorites: false,
                    }
                    .as_hash()
                    .as_ref(),
                );
            }
            In::UpdateFailure { errors } => {
                tx.send(&Out::Error(Patch::RemoveAll));
                for error in errors.iter() {
                    tx.send(&Out::Error(Patch::PushBack {
                        value: view! {
                            <li>{error}</li>
                        },
                    }));
                }
            }
            In::Logout => {
                let _ = store::delete_user();
                let _ = mogwai::utils::window()
                    .location()
                    .set_hash(Route::Home.as_hash().as_ref());
            }
        }
    }

    fn view(&self, tx: &Transmitter<In>, rx: &Receiver<Out>) -> ViewBuilder<HtmlElement> {
        builder! {
            <div class="settings-page">
                <div class="container page">
                    <div class="row">
                        <div class="col-md-6 offset-md-3 col-xs-12">
                            <h1 class="text-xs-center">"Your Settings"</h1>
                            <ul class="error-messages"
                                patch:children=rx.branch_filter_map(|msg| msg.errors())>
                            </ul>
                            <form>
                                <fieldset>
                                    <fieldset class="form-group">
                                        <input
                                            cast:type = HtmlInputElement
                                            post:build = tx.contra_map(|i: &HtmlInputElement| In::ImageInput(i.clone()))
                                            class="form-control"
                                            type="text"
                                            placeholder="URL of profile picture" />
                                    </fieldset>
                                    <fieldset class="form-group">
                                        <input
                                            cast:type = HtmlInputElement
                                            post:build = tx.contra_map(|i: &HtmlInputElement| In::UsernameInput(i.clone()))
                                            class="form-control form-control-lg"
                                            type="text"
                                            placeholder="Your Name" />
                                    </fieldset>
                                    <fieldset class="form-group">
                                        <textarea id="bio_input"
                                            cast:type = HtmlTextAreaElement
                                            post:build = tx.contra_map(|a: &HtmlTextAreaElement| In::BioInput(a.clone()))
                                            class="form-control form-control-lg"
                                            rows="8"
                                            placeholder="Short bio about you"
                                            >
                                        </textarea>
                                    </fieldset>
                                    <fieldset class="form-group">
                                        <input id="email_input"
                                            cast:type = HtmlInputElement
                                            post:build = tx.contra_map(|i: &HtmlInputElement| In::EmailInput(i.clone()))
                                            class="form-control form-control-lg"
                                             type="text"
                                             placeholder="Email" />
                                    </fieldset>
                                    <fieldset class="form-group">
                                        <input id="password_input"
                                            cast:type = HtmlInputElement
                                            post:build = tx.contra_map(|i: &HtmlInputElement| In::PasswordInput(i.clone()))
                                            class="form-control form-control-lg"
                                            type="password"
                                            placeholder="Password" />
                                    </fieldset>
                                    <button
                                        on:click=tx.contra_map(|_| In::Submit)
                                        class="btn btn-lg btn-primary pull-xs-right">
                                        "Update Settings"
                                    </button>
                                </fieldset>
                            </form>
                            <hr />
                            <button class="btn btn-outline-danger" on:click=tx.contra_map(|_| In::Logout)>
                                "Or click here to logout."
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
