pub mod book;
pub mod comment;
pub mod hu;
pub mod image;
pub mod layout;
pub mod message;
pub mod user;
pub mod widget;
pub mod xi;

use axum::{extract::State, routing::get, Router};

use axum_session::{Session, SessionPgPool};

use crate::fun::user::check_sudo;
use crate::md;
use layout::Html;
use maud::{html, PreEscaped};

use crate::http::{types::Page, Result, WebContext};

pub fn router() -> Router<WebContext> {
    Router::<WebContext>::new()
        .route("/", get(index))
        .route("/doc/help.html", get(doc_help))
        .route("/doc/about.html", get(doc_about))
        .route("/doc/contact.html", get(doc_contact))
        .route("/error/404", get(error_404))
}

async fn index(State(ctx): State<WebContext>, session: Session<SessionPgPool>) -> Result<Page> {
    let sudo = check_sudo(&session).await;
    let hu_top = hu::list_pub_top(&ctx, sudo).await?;
    let xi_top = xi::list_pub_top(&ctx, sudo).await?;
    let book_top = book::list_pub_top(&ctx, sudo).await?;
    let cat_name = hu::list_category_name(&ctx, false);
    let main = html! {
        div class="row row-cols-1" {
            div class="col col-md-9" {
                div class="border m-2 p-2" {
                    div class="bg-light p-2 border-bottom" {
                        a href={"/hu" } class="fs-5" {"最新文章"}
                    }
                    div class="p-2" {
                        (hu_top)
                    }
                }
                div class="border m-2 p-2" {
                    div class="bg-light p-2 border-bottom" {
                        a href={"/xi" } class="fs-5" {"最新微博"}
                    }
                    div class="p-2" {
                        (xi_top)
                    }
                }
                div class="border m-2 p-2" {
                    div class="bg-light p-2 border-bottom" {
                        a href={"/book" } class="fs-5" {"最新好书"}
                    }
                    div class="p-2" {
                        (book_top)
                    }
                }
            }
            div class="col col-md-3" {
                (cat_name)
            }
        }
    };
    Ok(Html::new("首页", main)
        .show_title(false)
        .icp(true)
        .highlight()
        .page(&ctx))
}

async fn error_404(State(ctx): State<WebContext>) -> Result<Page> {
    let main = html! {
        div {
            p { "提示错误 404：目标文件不存在" }
        }
    };
    Ok(Html::new("目标文件不存在", main).page(&ctx))
}

async fn doc_about(State(ctx): State<WebContext>) -> Result<Page> {
    let doc = std::fs::read_to_string("htdocs/docs/about.md")
        .map_err(|_| anyhow::anyhow!("read about.md error"))?;
    let html = md::to_html(doc.as_str());
    let main = html! {
        div class="shadow-lg p-5 mb-5 bg-body rounded md" {
            (PreEscaped(html))
        }
    };
    Ok(Html::new("关于我们", main).page(&ctx))
}

async fn doc_help(State(ctx): State<WebContext>) -> Result<Page> {
    let doc = std::fs::read_to_string("htdocs/docs/help.md")
        .map_err(|_| anyhow::anyhow!("read help.md error"))?;
    let html = md::to_html(doc.as_str());
    let main = html! {
        div class="shadow-lg p-5 mb-5 bg-body rounded md" {
            (PreEscaped(html))
        }
    };
    Ok(Html::new("帮助", main).page(&ctx))
}

async fn doc_contact(State(ctx): State<WebContext>) -> Result<Page> {
    let doc = std::fs::read_to_string("htdocs/docs/contact.md")
        .map_err(|_| anyhow::anyhow!("read help.md error"))?;
    let html = md::to_html(doc.as_str());
    let main = html! {
        div class="shadow-lg p-5 mb-5 bg-body rounded md" {
            (PreEscaped(html))
        }
    };
    Ok(Html::new("联系方法", main).page(&ctx))
}
