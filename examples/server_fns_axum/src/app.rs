use leptos::prelude::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <meta name="color-scheme" content="dark light" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <link rel="stylesheet" id="leptos" href="/pkg/server_fns_axum.css" />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let outer_r = OnceResource::new_blocking(async move {
        #[cfg(feature = "ssr")]
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok::<_, ()>(())
    });

    let inner_view = move || {
        Suspend::new(async move {
            let _ = outer_r.await;

            view! { <InnerComponent /> }
        })
    };

    view! {
        <main>
            <Suspense>{inner_view}</Suspense>
        </main>
    }

    // FOR WORKING CASE, REPLACE THIS APP BODY WITH: `view! { <InnerComponent /> }`
}

#[component]
pub fn InnerComponent() -> impl IntoView {
    let inner_r = OnceResource::new_blocking(async move {
        let cookie_name =
            uuid::Uuid::new_v4().to_string().replace("-", "")[0..5].to_string();
        let cookie_value =
            uuid::Uuid::new_v4().to_string().replace("-", "")[0..5].to_string();

        #[cfg(feature = "ssr")]
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let _ = s_set_cookie(cookie_name.clone(), cookie_value.clone()).await;
        (cookie_name, cookie_value)
    });

    let getter = Action::new(move |(name,): &(String,)| {
        let name = name.clone();
        async move {
            s_get_cookie(name)
                .await
                .unwrap()
                .unwrap_or_else(|| "NO COOKIE SET".to_string())
        }
    });

    view! {
        <Suspense>
            {move || {
                Suspend::new({
                    async move {
                        let (name, value) = inner_r.await;
                        let name = StoredValue::new(name);
                        let value = StoredValue::new(value);
                        view! {
                            <div>
                                <button
                                    type="button"
                                    on:click=move |_e| {
                                        getter.dispatch((name.get_value(),));
                                    }
                                >
                                    {"Check cookie"}
                                </button>
                                <p>
                                    {move || {
                                        view! {
                                            {format!(
                                                "Expecting cookie of {}={}",
                                                name.get_value(),
                                                value.get_value(),
                                            )}
                                            <br />
                                            {format!(
                                                "cookie: {}",
                                                getter
                                                    .value()
                                                    .get()
                                                    .unwrap_or_else(|| {
                                                        "click 'Check cookie' to see".to_string()
                                                    }),
                                            )}
                                        }
                                    }}
                                </p>
                            </div>
                        }
                    }
                })
            }}
        </Suspense>
    }
}

#[server]
pub async fn s_set_cookie(
    name: String,
    value: String,
) -> Result<(), ServerFnError> {
    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        opts.insert_header(
            http::header::SET_COOKIE,
            http::header::HeaderValue::from_str(&format!("{}={}", name, value))
                .unwrap(),
        )
    } else {
        println!("No response options found");
    }
    Ok(())
}

#[server]
pub async fn s_get_cookie(
    name: String,
) -> Result<Option<String>, ServerFnError> {
    if let Some(parts) = leptos::prelude::use_context::<http::request::Parts>()
    {
        let cookies = axum_extra::extract::cookie::CookieJar::from_headers(
            &parts.headers,
        );
        println!("COOKIES: {:#?}", cookies);
        let found = cookies.get(&name).map(|cookie| cookie.value().to_string());
        println!("For name: {}, FOUND: {:#?}", name, found);
        Ok(found)
    } else {
        println!("No request parts found");
        Ok(None)
    }
}
