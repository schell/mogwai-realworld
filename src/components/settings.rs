use mogwai::prelude::*;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

use crate::{
    api::{self, User, UserUpdate},
    route::Route,
    store,
    widgets::{TextInput, TextInputIn},
};

/// The settings UI component.
pub struct Settings {
    o_user: Option<User>,
    pic_input: Gizmo<TextInput<HtmlInputElement>>,
    name_input: Gizmo<TextInput<HtmlInputElement>>,
    bio_input: Gizmo<TextInput<HtmlTextAreaElement>>,
    email_input: Gizmo<TextInput<HtmlInputElement>>,
    password_input: Gizmo<TextInput<HtmlInputElement>>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings = Settings {
            pic_input: Gizmo::from(TextInput::new("", "URL of profile picture")),
            name_input: Gizmo::from(TextInput::new("", "Your name")),
            bio_input: Gizmo::from(TextInput::new("", "Short bio about you")),
            email_input: Gizmo::from(TextInput::new("", "Your email")),
            password_input: Gizmo::from(TextInput::new("", "Your password")),
            o_user: None,
        };
        if let Some(user) = store::read_user().ok() {
            settings.set_user(user);
        }
        settings
    }
}

impl Settings {
    fn set_user(&mut self, user: User) {
        if let Some(image) = user.image.as_ref() {
            self.pic_input.send(&TextInputIn::SetValue(image.to_string()));
        }
        if let Some(bio) = user.bio.as_ref() {
            self.bio_input.send(&TextInputIn::SetValue(bio.to_string()));
        }
        self.name_input.send(&TextInputIn::SetValue(user.username.clone()));
        self.email_input.send(&TextInputIn::SetValue(user.email.clone()));
        self.o_user = Some(user);
    }
}

#[derive(Clone)]
pub enum In {
    GotUser(User),
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

    fn bind(&self, sub: &Subscriber<In>) {
        if let Some(user) = self.o_user.as_ref() {
            let user = user.clone();
            sub.send_async(async move {
                let user = api::get_user(&user.token)
                    .await
                    .unwrap_or_else(|e| panic!("could not get user: {}", e));
                In::GotUser(user)
            });
        }
    }

    fn update(&mut self, msg: &In, tx: &Transmitter<Out>, sub: &Subscriber<In>) {
        match msg {
            In::GotUser(user) => {
                self.set_user(user.clone());
                let _ = store::write_user(user);
            }
            In::Submit => {
                let email = Some(self.email_input.state.borrow().value.clone());
                let username = Some(self.name_input.state.borrow().value.clone());
                let bio = Some(self.bio_input.state.borrow().value.clone());
                let image = Some(self.pic_input.state.borrow().value.clone());
                let password = {
                    let pass = self.password_input.state.borrow().value.clone();
                    if pass.is_empty() {
                        None
                    } else {
                        Some(pass)
                    }
                };

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
                                        {self.pic_input.view_builder()}
                                    </fieldset>
                                    <fieldset class="form-group">
                                        {self.name_input.view_builder()}
                                    </fieldset>
                                    <fieldset class="form-group">
                                        {self.bio_input.view_builder()}
                                    </fieldset>
                                    <fieldset class="form-group">
                                        {self.email_input.view_builder()}
                                    </fieldset>
                                    <fieldset class="form-group">
                                        {self.password_input.view_builder()}
                                    </fieldset>
                                    <button
                                        on:click=tx.contra_map(|_| In::Submit)
                                        class="btn btn-lg btn-primary pull-xs-right">
                                        "Update Settings"
                                    </button>
                                </fieldset>
                            </form>
                            <hr />
                            <button
                                class="btn btn-outline-danger"
                                on:click=tx.contra_map(|_| In::Logout)>
                                "Or click here to logout."
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
