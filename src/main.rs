mod todo_context;

use dioxus::prelude::*;
use crate::todo_context::{TodoState, Todo, save_todos, load_todos};
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
    let categories = use_signal(|| vec!["Work".to_string(), "Personal".to_string(), "Shopping".to_string()]);
    let mut selected_category: Signal<Option<String>> = use_signal(|| None::<String>);
    let mut filter_category = use_signal(|| None::<String>);
    let mut edited_todo_cat = use_signal(|| None::<String>);

    rsx! {
        div {
            h1 { "Todo List" }
            input {
                class: "todo-input",
                placeholder: "Enter a todo...",
                value: "{new_todo}",
                oninput: move |evt| new_todo.set(evt.value().clone()),
            }
            select {
                onchange: move |evt| selected_category.set(evt.value().clone().into()),
                option { selected: if selected_category.read().is_none() {true} else {false}, value: "", "Select Category" }
                for category in categories.read().iter() {
                    option { value: "{category}", "{category}" }
                }
            }
            button {
                onclick: move |_| {
                    if !new_todo.read().is_empty() && !selected_category.read().is_none() {
                        todos.write().todos.push(Todo {
                           text: new_todo.read().clone(),
                           category: selected_category.read().clone(),
                        });
                        new_todo.set(String::new());
                        selected_category.set(None);
                        save_todos(&todos.read().todos);
                    }
                },
                "Add Todo"
            }
        }
        div {
            select {
                class: "filter-select",
                onchange: move |evt| filter_category.set(if evt.value().is_empty() { None } else { Some(evt.value().clone()) }),
                option { value: String::new(), "All Categories" }
                for category in categories.read().iter() {
                    if !filter_category.read().is_none() && *category == filter_category.read().clone().unwrap() {
                        option { value: "{category}", selected: true, "{category}" }
                    } else {
                        option { value: "{category}", "{category}" }
                    }
                }
            }
            ul {{
                let todos_vec = todos.read().todos.clone();
                todos_vec.into_iter().filter(|todo| { 
                    if !filter_category.read().is_none() { 
                        filter_category.read().as_ref() == todo.category.as_ref()
                    } else {
                        true
                    } 
                }).enumerate().map(|(ix, todo)| {
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
                                        value: "{todo.text}",
                                        oninput: move |evt| edited_todo_text.set(evt.value().clone()),
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Escape {
                                                editing_todo_ix.set(None);
                                            }
                                        }
                                    },
                                    select {
                                        onchange: move |evt| {
                                            edited_todo_cat.set(Some(evt.value().clone()));
                                        },
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Escape {
                                                editing_todo_ix.set(None);
                                            }
                                        },
                                        option { value: "", "No Category" }
                                        for category in categories.read().iter() {
                                            if *category == todo.category.clone().unwrap() {
                                                option { value: "{category}", selected: true, "{category}" }
                                            } else {
                                                option { value: "{category}", "{category}" }
                                            }
                                        }
                                    }
                                    button {
                                        onclick: move |_| {
                                            todos.write().todos[ix].text = edited_todo_text.read().clone();
                                            todos.write().todos[ix].category = edited_todo_cat.read().clone();
                                            editing_todo_ix.set(None);
                                            save_todos(&todos.read().todos);
                                        },
                                        "Save"
                                    }
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
                                            edited_todo_text.set(todo.text.clone());
                                            edited_todo_cat.set(todo.category.clone());
                                            editing_todo_ix.set(Some(ix));
                                        }, 
                                        "{todo.text} - {todo.category.clone().unwrap()}"
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
