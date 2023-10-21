use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use html2maud::*;

#[function_component]
pub fn App() -> Html {
    let html_src = "<html>
<header><title>Sample HTML</title></header>
<body>
<div class='container' id='main'>
    <div class='col-12'>
        <span class='badge badge-success'>Hello html</span>
        <input disabled=true onchange='alert(this.value)' />
    </div>
</div>
</body>
</html>";
    let html = use_state(|| html_src.to_owned());
    let maud = use_state(|| html2maud(html.as_str()));

    let oninput = Callback::from({
        let html = html.clone();
        let maud = maud.clone();

        move |input_event: InputEvent| {
            let target: HtmlTextAreaElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            html.set(target.value());
            maud.set(html2maud(&target.value()));
        }
    });

    html! {
        <div>
            <h2>{"Html2Maud converter"}</h2>
            <span class="info-badge">
                {"This converter runs entirely in the browser, no html is sent to a server"}
            </span>
            <div class="container">
                <textarea {oninput} autofocus=true value={html.as_str().to_owned()} />
                <textarea readonly=true value={maud.as_str().to_owned()} />
            </div>
        </div>
    }
}
