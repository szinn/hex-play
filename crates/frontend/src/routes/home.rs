use dioxus::prelude::*;

use crate::user::{get_permissions, get_user_name, login, logout};

#[component]
pub(crate) fn Home() -> Element {
    let mut login = use_action(login);
    let mut user_name = use_action(get_user_name);
    let mut permissions = use_action(get_permissions);
    let mut logout = use_action(logout);

    let fetch_new = move |_| async move {
        user_name.call().await;
        permissions.call().await;
    };

    rsx! {
        div { class: "p-6 space-y-4",
            h1 { class: "text-2xl font-bold text-gray-900", "Home" }
            div { class: "flex gap-3",
                button {
                    class: "px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 text-sm",
                    onclick: move |_| async move {
                        login.call().await;
                    },
                    "Login Test User"
                }
                button {
                    class: "px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 text-sm",
                    onclick: move |_| async move {
                        logout.call().await;
                    },
                    "Logout"
                }
                button {
                    class: "px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 text-sm",
                    onclick: fetch_new,
                    "Fetch User Info"
                }
            }

            pre { class: "bg-gray-100 p-3 rounded text-sm", "Logged in: {login.value():?}" }
            pre { class: "bg-gray-100 p-3 rounded text-sm", "User name: {user_name.value():?}" }
            pre { class: "bg-gray-100 p-3 rounded text-sm", "Permissions: {permissions.value():?}" }
        }
    }
}
