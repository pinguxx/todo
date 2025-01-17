mod todo_context;

use dioxus::prelude::*;
use crate::todo_context::{TodoState, save_todos, load_todos};
use std::rc::Rc;
use crate::dioxus_elements::MountedData;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(TodoState {
        todos: load_todos(),
    }));
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Todos {}

    }
}

#[component] 
fn Dialog(on_close: EventHandler<()>, on_save: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "modal-overlay",
            onclick:  move |_| on_close.call(()),
            div {
                class: "modal-content",
                onclick: |e| e.stop_propagation(),
                p { "Are you sure?" },
                button {
                    onclick: move |_| on_save.call(()),
                    "Yes"
                }
                button {
                    onmounted: move |element| async move {
                        let _ = element.data().set_focus(true).await;
                    },
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}

#[component]
pub fn Todos() -> Element {
    //let mut todos = use_signal(|| vec![]);
    let mut todos = use_context::<Signal<TodoState>>();
    let mut new_todo = use_signal(|| String::new());
    let mut editing_todo_ix = use_signal(|| None::<usize>);
    let mut edited_todo_text = use_signal(|| String::new());
    let mut show_dialog = use_signal(|| false);
    let mut input_ref = use_signal(|| None::<Rc<MountedData>>);

    rsx! {
        div {
            h1 { "Todo List" }
            input {
                class: "todo-input",
                placeholder: "Enter a todo...",
                value: "{new_todo}",
                oninput: move |evt| new_todo.set(evt.value().clone()),
            }
            button {
                onclick: move |_| {
                    if !new_todo.read().is_empty() {
                        todos.write().todos.push(new_todo.read().clone());
                        new_todo.set(String::new());
                        save_todos(&todos.read().todos);
                    }
                },
                "Add Todo"
            }
            ul {{
                let todos_vec = todos.read().todos.clone();
                todos_vec.into_iter().enumerate().map(|(ix, todo)| {
                    {
                        if *editing_todo_ix.read() == Some(ix) {
                            rsx! {
                                li {
                                    key: "{ix}",
                                    input {
                                        onmounted: move |element| async move {
                                            input_ref.set(Some(element.data().clone()));
                                            let _ = element.data().set_focus(true).await;
                                        },
                                        value: "{edited_todo_text}",
                                        oninput: move |evt| edited_todo_text.set(evt.value().clone()),
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                todos.write().todos[ix] = edited_todo_text.read().clone();
                                                editing_todo_ix.set(None);
                                                save_todos(&todos.read().todos);
                                            }
                                            if evt.key() == Key::Escape {
                                                editing_todo_ix.set(None);
                                            }
                                        }
                                    },
                                    button {
                                        onclick: move |_| show_dialog.set(true),
                                        "Delete"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                li {
                                    key: "{ix}",
                                    span {
                                        class: "edit",
                                        onclick: move |_| {
                                            edited_todo_text.set(todo.clone());
                                            editing_todo_ix.set(Some(ix));
                                        }, 
                                        "{todo}"
                                    }
                                }
                            }
                        }
                    }  
                })
            }}
            if *show_dialog.read() {
                Dialog {
                    on_close: move |_| {
                        if let Some(node) =  input_ref.read().as_ref().cloned() {
                            spawn(async move {
                                let _ = node.set_focus(true).await;
                            });
                        };
                        show_dialog.set(false);
                    },
                    on_save: move |_| {
                        todos.write().todos.remove(editing_todo_ix.read().unwrap());
                        save_todos(&todos.read().todos);
                        editing_todo_ix.set(None);
                        show_dialog.set(false);
                    },
                }
            }
        }
    }
}
