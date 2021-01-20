use yew::prelude::*;
use yew_router::prelude::RouterAnchor;

use crate::routes::AppRoute;

pub(crate) fn footer() -> Html {
    html! {
        <div class="container">
            <div class="row">
                <div class="col-12 col-md-6">
                    { "© 2021 White-Green All rights reserved." }
                </div>
                <div class="col-12 col-md-6 text-md-right">
                    <RouterAnchor<AppRoute> route={ AppRoute::License } classes="nav-link" >{ "License" }</RouterAnchor<AppRoute>>
                </div>
            </div>
        </div>
    }
}
