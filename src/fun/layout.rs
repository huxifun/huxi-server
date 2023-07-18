//! 模板

use maud::{html, Markup, DOCTYPE};

use crate::http::{types::Page, WebContext};
use std::fs;

use super::user::SessUser;

fn top_nav(ctx: &WebContext, path: Option<&str>) -> Markup {
    let path = path.unwrap_or("");
    let active = |url: &str| -> &str {
        if path.starts_with(url) {
            "nav-link active border-bottom border-secondary border-3"
        } else {
            "nav-link"
        }
    };
    let my_name = ctx.config.host.my_name.as_str();
    let logo_name = ctx.config.host.name.as_str();

    html!(
        div style="background-color: #e3f2fd;" {
                div class="container" {
        nav class="navbar navbar-expand-md "{
            div class="container-fluid " {
                a class="navbar-brand" href="/" {
                    img src=(ctx.config.host.logo) width="50" class="me-2";
                    (logo_name)
                }
                button class="navbar-toggler" type="button" data-bs-toggle="collapse"
                    data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent"
                    aria-expanded="false" aria-label="Toggle navigation" {
                    span class="navbar-toggler-icon" {}
                }
                div class="collapse navbar-collapse" id="navbarSupportedContent" {
                    ul class="navbar-nav me-auto mb-2 mb-lg-0" {
                        li class="nav-item" {
                            a class="nav-link" href="/" {"首页"}
                        }
                        li class="nav-item" {
                            a class={ (active("/xi")) } href="/xi" {"微博" }
                        }
                        li class="nav-item" {
                            a class={ (active("/hu")) } href="/hu" {"文章" }
                        }
                        li class="nav-item" {
                            a class={ (active("/book")) } href="/book" {"好书" }
                        }

                        li class="nav-item" {
                            a class={ (active("/my")) } href="/my/hx" {(my_name) }
                        }

                        li class="nav-item dropdown" {
                            a class="nav-link dropdown-toggle" href="/my/hx" id="navbarDropdown"
                                role="button" data-bs-toggle="dropdown" aria-expanded="false" {
                                "更多"
                            }
                            ul class="dropdown-menu shadow p-3 mb-5 bg-body rounded" aria-labelledby="navbarDropdown" {
                                li {a class="dropdown-item" href="/my/xi" {"我的微博"}}
                                li {a class="dropdown-item" href="/my/hu" {"我的文章"}}
                                li {a class="dropdown-item" href="/my/book" {"我的好书"}}
                                li {a class="dropdown-item" href="/my/inbox" {"我的信箱"}}
                                li {hr class="dropdown-divider"; }
                                li {a class="dropdown-item" href="/my/xi/add" {"写微博"}}
                                li {a class="dropdown-item" href="/my/hu/add" {"写文章"}}
                                li {a class="dropdown-item" href="/my/book/add" {"推荐好书"}}
                            }
                        }
                    }


                    form class="d-flex" role="search" action="/hu/search" method="get" {
                        input class="form-control me-2" type="search" placeholder="Search" aria-label="Search" name="key";
                        button class="btn btn-outline-success flex-shrink-0" type="submit" {"搜索文章"}
                    }

                }
                span id="login" class="flex-shrink-0 p-2" {
                    a href="/user/reg" class="btn btn-primary mx-1" {"注册" }
                    a href="/user/login" class="btn btn-primary" {"登录" }
                }
            }
        }
                }
        }
    )
}
pub struct Html<'a> {
    title: String,
    main: Markup,
    keywords: Option<String>,
    description: Option<String>,
    highlight: bool,
    mde: bool, // markdown editor
    path: Option<&'a str>,
    icp: bool,
    my_huxi: bool,
    user_name: Option<&'a str>,
    user_id: i32,
    show_title: bool,
    sub_nav: Option<&'a str>,
    head: String,
}
impl<'a> Html<'a> {
    pub fn new(title: &str, main: Markup) -> Html {
        Html {
            main,
            title: title.to_string(),
            keywords: None,
            description: None,
            highlight: false,
            my_huxi: false,
            icp: false,
            path: None,
            user_name: None,
            user_id: 0,
            mde: false,
            show_title: true,
            sub_nav: None,
            head: "/img/head.png".to_string(),
        }
    }
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    pub fn main(mut self, main: Markup) -> Self {
        self.main = main;
        self
    }
    pub fn keywords(mut self, k: Option<String>) -> Self {
        self.keywords = k;
        self
    }
    pub fn description(mut self, d: Option<String>) -> Self {
        self.description = d;
        self
    }
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }
    pub fn mde(mut self) -> Self {
        self.mde = true;
        self
    }
    pub fn path(mut self, path: Option<&'a str>) -> Self {
        self.path = path;
        self
    }
    pub fn my_huxi(mut self, user: &'a SessUser) -> Self {
        self.my_huxi = true;
        self.user_name = Some(user.name.as_str());
        self.user_id = user.id;
        self
    }
    fn get_head(&self, ctx: &WebContext) -> String {
        let mut src = self.head.clone();
        let img_path1 = format!("{}/s-{}.jpg", ctx.config.user.upload_path, self.user_id);
        let img_path2 = format!("{}/s-{}.png", ctx.config.user.upload_path, self.user_id);
        let img_path3 = format!("{}/s-{}.gif", ctx.config.user.upload_path, self.user_id);
        if fs::metadata(img_path1).is_ok() {
            src = format!("{}/s-{}.jpg", ctx.config.user.public_url, self.user_id);
        } else if fs::metadata(img_path2).is_ok() {
            src = format!("{}/s-{}.png", ctx.config.user.public_url, self.user_id);
        } else if fs::metadata(img_path3).is_ok() {
            src = format!("{}/s-{}.gif", ctx.config.user.public_url, self.user_id);
        }
        src
    }
    pub fn icp(mut self, i: bool) -> Self {
        self.icp = i;
        self
    }

    pub fn sub_nav(mut self, nav: Option<&'a str>) -> Self {
        self.sub_nav = nav;
        self
    }
    pub fn my_nav_item(&self, url: &str, txt: &str) -> Markup {
        let path = self.path.map_or("", |v| v);
        let active = path.starts_with(url);
        html!(
            li class="nav-item mybar" {
                @if active {
                    a href=(url) class="nav-link active border p-2 m-1 h5 bg-secondary text-white shadow mb-2 rounded" aria-current="page"  {(txt)}
                } @else  {
                    a href=(url) class="nav-link border p-2 m-1 h5 bg-light shadow mb-2 rounded" {(txt)}
                }
            }
        )
    }
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }
    pub fn page(self, ctx: &WebContext) -> Page {
        Page(html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta http-equiv="X-UA-Compatible" content="IE=edge";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (&self.title) " -- " (ctx.config.host.name)}
                @if let Some(ref des) = self.description {
                    meta name="description" content={(des)};
                }
                link rel="icon" href="/img/favicon.ico" type="image/x-icon";
                //link rel="stylesheet" href="/css/tailwind.css";
                link rel="stylesheet" href="/css/bootstrap.min.css";
                //script src="/js/index.min.js" {}
                @if self.mde {
                    link rel="stylesheet" href="/css/easymde.min.css";
                    script src="/js/easymde.min.js" {}
                }
                @if self.highlight {
                    link rel="stylesheet" href="/css/hl/github.css";
                    script src="/js/hl/highlight.min.js" {}
                    script {"hljs.highlightAll();"}
                }
                link rel="stylesheet" href="/css/main.css";
            }
            body {
                (top_nav(ctx, self.path))
                div class="container mx-auto" {
                //导航
                    div class="mx-auto justify-center border-2 border-indigo-600" {
                        @if self.my_huxi {
                            // my main start
                            div class="row p-2 m-2" {
                                div class="col-md-3 border border-primary p-2 shadow mb-5 bg-body rounded  order-2 order-md-1" {
                                    div class="text-center" {
                                        img class="img-thumbnail" src={(self.get_head(ctx))};
                                        h4 class="text-center" {(self.user_name.unwrap_or(""))}
                                    }
                                    ul class="nav flex-column text-center mb-4" {
                                        (self.my_nav_item("/my/hu", "文章"))
                                        (self.my_nav_item("/my/xi", "微博"))
                                        (self.my_nav_item("/my/book", "好书"))
                                        (self.my_nav_item("/my/image", "图片"))
                                        (self.my_nav_item("/my/inbox", "收信箱"))
                                        (self.my_nav_item("/my/outbox", "发信箱"))
                                        (self.my_nav_item("/my/info", "设置"))
                                    }
                                    @let pp = self.path.map_or("", |v| v);
                                    @if pp.starts_with("/my/xi") {
                                        (crate::fun::xi::list_category_name(ctx, true))
                                    }
                                    @if pp.starts_with("/my/book") {
                                        (crate::fun::book::list_category_name(ctx, true))
                                    }
                                    @if pp.starts_with("/my/hu") {
                                        (crate::fun::hu::list_category_name(ctx, true))
                                    }
                                }
                                div id="start" class="col-md-9 order-1 order-md-2"{
                                    @if let Some(nav) = self.sub_nav {
                                        nav style="--bs-breadcrumb-divider: '>';" aria-label="breadcrumb" {
                                            ol class="breadcrumb" {
                                                li class="breadcrumb-item" {
                                                    a href="/" {"Home"}
                                                }
                                                li class="breadcrumb-item" {
                                                    a href="/my/hx" {(ctx.config.host.my_name)}
                                                }
                                                li class="breadcrumb-item active" aria-current="page" {
                                                    (nav)
                                                }
                                            }
                                        }
                                    }


                                    @if self.show_title {
                                        h4 class="text-center bg-light p-2 m-3" { (&self.title) }
                                    }
                                    (self.main)
                                }
                            }
                            // end my main
                        } @else {
                            @if self.show_title {
                                h4 class="text-center m-3 bg-light p-2" { (&self.title) }
                            }
                            (self.main)
                        }
                    }
                }
                div class="bg-light" {
                    div class="container" {
                        div class="my-5 py-3 text-muted text-center text-small bg-light" {
                            ul class="list-inline" {
                                li class="list-inline-item" { a href="/doc/about.html" {"关于我们" } }
                                li class="list-inline-item" {" - " }
                                li class="list-inline-item" { a href="/doc/help.html" {"帮助" } }
                                li class="list-inline-item" {" - " }
                                li class="list-inline-item" { a href="/doc/contact.html" {"联系方法" } }
                                li class="list-inline-item" {" - " }
                                li class="list-inline-item" { a href="/my/outbox/new?to=huxi" {"留言" } }
                            }
                            p {
                                img src="/img/logo/huxi.png" class="mx-2";
                                (ctx.config.host.copyright)
                                    @if self.icp {
                                        a href="https://beian.miit.gov.cn/" {span class="mx-3" { "(" (ctx.config.host.icp) ")"}}
                                    }
                            }
                        }
                    }
                }
                script type="text/javascript" src="/js/bootstrap.bundle.min.js" {}
                script type="text/javascript" src="/js/jquery-3.6.0.min.js" {}
                script type="text/javascript" src="/js/main.js" {}
        }
        }
        })
    }
}

pub fn split(left: Markup, right: Markup) -> Markup {
    html!(
        div class="row row-cols-1" {
            div class="col col-md-9" {
                (left)
            }
            div class="col col-md-3 bg-light p-2" {
                (right)
            }
        }
    )
}
pub fn vsplit(up: Markup, down: Markup) -> Markup {
    html!(
        div class="" {
            div class="border m-2 p-1" {
                (up)
            }
            div class="" {
                (down)
            }
        }
    )
}
