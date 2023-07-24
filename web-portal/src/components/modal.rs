use leptos::*;

pub const ADD_MODAL_TARGET: &str = "#modals";
pub const ADD_MODAL_SWAP: &str = "beforeend";

#[component]
pub fn EditModal<S1, S2, S3, IV>(
    cx: Scope,
    id: S1,
    title: S2,
    #[prop(optional)] size: ModalSize,
    form: IV,
    patch_url: S3,
    #[prop(optional)] target: Option<String>,
) -> impl IntoView
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    IV: IntoView,
{
    let id = id.into();
    let patch_url = patch_url.into();
    view! { cx,
        <Modal
            id=id.clone()
            title=title
            size=size
            body=view! { cx,
                <form id="editForm">{form}</form>
            }
            buttons=view! { cx,
                <button
                    type="button"
                    class="btn btn-secondary"
                    onclick="closeModal(this)"
                    data-em-modal=id
                    hx-patch=patch_url
                    hx-include="#editForm"
                    hx-target=target
                >"Confirm"</button>
            }/>
    }
}

#[component]
pub fn CreateModal<S1, S2, S3, IV>(
    cx: Scope,
    id: S1,
    title: S2,
    #[prop(optional)] size: ModalSize,
    form: IV,
    post_url: S3,
    #[prop(optional)] target: Option<String>,
) -> impl IntoView
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    IV: IntoView,
{
    let id = id.into();
    let post_url = post_url.into();
    view! { cx,
        <Modal
            id=id.clone()
            title=title
            size=size
            body=view! { cx,
                <form id="createForm">{form}</form>
            }
            buttons=view! { cx,
                <button
                    type="button"
                    class="btn btn-secondary"
                    onclick="closeModal(this)"
                    data-em-modal=id
                    hx-post=post_url
                    hx-include="#createForm"
                    hx-target=target
                >"Confirm"</button>
            }/>
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub enum ModalSize {
    Small,
    #[default]
    Default,
    Large,
    ExtraLarge,
}

impl ModalSize {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "modal-sm",
            Self::Default => "",
            Self::Large => "modal-lg",
            Self::ExtraLarge => "modal-xl",
        }
    }
}

#[component]
pub fn Modal<S1, S2, IV, IV2>(
    cx: Scope,
    id: S1,
    title: S2,
    #[prop(optional)] size: ModalSize,
    body: IV,
    buttons: IV2,
) -> impl IntoView
where
    S1: Into<String>,
    S2: Into<String>,
    IV: IntoView,
    IV2: IntoView,
{
    let id = id.into();
    view! { cx,
        <div id=format!("{id}-backdrop") class="modal-backdrop fade show" style="display: block;"></div>
        <div id=id.clone() class="modal fade show" tabindex="-1" style="display:block;">
            <div class=format!("modal-dialog {} modal-dialog-centered", size.as_str())>
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">{title.into()}</h5>
                    </div>
                    <div class="modal-body">
                        <p>{body}</p>
                    </div>
                    <div class="modal-footer">
                        {buttons}
                        <button type="button" class="btn btn-secondary" onclick="closeModal(this)" data-em-modal=id>"Close"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
