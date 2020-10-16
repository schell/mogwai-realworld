use mogwai::prelude::*;

use crate::{
    api::{self, Article, Articles, User, UserProfile},
    store,
};

fn article_builder(article: &Article) -> ViewBuilder<HtmlElement> {
    builder! {
        <div class="article-preview">
            <div class="article-meta">
                <a href=""><img src=&article.author.image /></a>
                <div class="info">
                    <a href="" class="author">{&article.author.username}</a>
                    <span class="date">"January 20th"</span>
                </div>
                <button class="btn btn-outline-primary btn-sm pull-xs-right">
                    <i class="ion-heart"></i>
                    {format!(" {}", article.favorites_count)}
                </button>
            </div>
            <a href="" class="preview-link">
                <h1>{&article.title}</h1>
                <p>{&article.description}</p>
                <span>"Read more..."</span>
            </a>
        </div>
    }
}

pub struct Profile {
    pub username: String,
    pub is_favorites: bool,
}

#[derive(Clone)]
pub enum In {
    Articles(Articles),
    Profile(UserProfile),
}

#[derive(Clone)]
pub enum Out {
    ArticlePatch(Patch<View<HtmlElement>>),
    Profile(UserProfile),
}

impl Out {
    fn articles(&self) -> Option<Patch<View<HtmlElement>>> {
        if let Out::ArticlePatch(patch) = self {
            Some(patch.clone())
        } else {
            None
        }
    }

    fn user(&self) -> Option<UserProfile> {
        if let Out::Profile(user) = self {
            Some(user.clone())
        } else {
            None
        }
    }
}

impl Component for Profile {
    type ModelMsg = In;
    type ViewMsg = Out;
    type DomNode = HtmlElement;

    fn bind(&self, sub: &Subscriber<Self::ModelMsg>) {
        let username = self.username.clone();
        let o_token = store::read_user().ok().map(|u| u.token);
        sub.send_async(async move {
            let profile = api::get_profile(&username, o_token.as_deref())
                .await
                .unwrap_or_else(|e| panic!("could not get profile '{}': {}", username, e));
            In::Profile(profile)
        });
        let username = self.username.clone();
        let o_token = store::read_user().ok().map(|u| u.token);
        sub.send_async(async move {
            let articles =
                api::get_articles(Some(&username), None, None, None, None, o_token.as_deref())
                    .await
                    .unwrap_or_else(|e| panic!("could not fetch articles '{}': {}", username, e));
            In::Articles(articles)
        })
    }

    fn update(
        &mut self,
        msg: &Self::ModelMsg,
        tx: &Transmitter<Self::ViewMsg>,
        sub: &Subscriber<Self::ModelMsg>,
    ) {
        match msg {
            In::Articles(articles) => {
                tx.send(&Out::ArticlePatch(Patch::RemoveAll));
                for article in articles.articles.iter() {
                    tx.send(&Out::ArticlePatch(Patch::PushBack {
                        value: View::from(article_builder(article)),
                    }));
                }
            }
            In::Profile(user_profile) => {
                tx.send(&Out::Profile(user_profile.clone()));
            }
        }
    }

    fn view(
        &self,
        tx: &Transmitter<Self::ModelMsg>,
        rx: &Receiver<Self::ViewMsg>,
    ) -> ViewBuilder<HtmlElement> {
        builder! {
            <div class="profile-page">
                <div class="user-info">
                    <div class="container">
                        <div class="row">
                            <div class="col-xs-12 col-md-10 offset-md-1">
                                <img
                                    src=rx.branch_filter_map(|msg| msg.user().map(|u| u.image))
                                    class="user-img" />
                                <h4>{rx.branch_filter_map(|msg| msg.user().map(|u| u.username))}</h4>
                                <p>
                                    "Some BS about some stuff."
                                </p>
                                <button class="btn btn-sm btn-outline-secondary action-btn">
                                    <i class="ion-plus-round"></i>
                                    {
                                        rx.branch_filter_map(|msg| {
                                            msg.user().map(|u| format!(" Follow {}", u.username))
                                        })
                                    }
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="container">
                    <div class="row">
                        <div class="col-xs-12 col-md-10 offset-md-1">
                            <div class="articles-toggle">
                                <ul class="nav nav-pills outline-active">
                                    <li class="nav-item">
                                        <a class="nav-link active" href="">"My Articles"</a>
                                    </li>
                                    <li class="nav-item">
                                        <a class="nav-link" href="">"Favorited Articles"</a>
                                    </li>
                                </ul>
                            </div>
                            <slot patch:children=rx.branch_filter_map(|msg| msg.articles())>
                            </slot>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
