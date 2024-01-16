extern crate proc_macro;

use core::panic;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Item, ItemImpl, ItemMod, ItemStruct, Lit, Meta, MetaNameValue,
    NestedMeta,
};

const CMD_ATTR: &str = "handler";

struct CommandAttribute {
    command: Option<String>,
    chat_mode_start: Option<bool>,
    chat_mode_exit: Option<bool>,
    llm_request: Option<bool>,
}

#[proc_macro_attribute]
pub fn bot_commands(_args: TokenStream, input: TokenStream) -> TokenStream {
    let module = parse_macro_input!(input as ItemMod);
    let mut commands = Vec::new();
    let mut new_items = Vec::new();
    let mut chat_start_cmd: Option<String> = None;
    let mut chat_exit_cmd: Option<String> = None;
    let mut llm_request_cmd: Option<String> = None;

    let (brace, items) = &module.content.as_ref().unwrap();
    for item in items {
        match item {
            syn::Item::Fn(func) => {
                let CommandAttribute {
                    command: cmd,
                    chat_mode_start: chat_start,
                    chat_mode_exit: chat_exit,
                    llm_request: llm_req,
                } = get_command_attribute(&func.attrs);
                if let Some(command) = cmd {
                    let func_name = &func.sig.ident;
                    let command_name = command;
                    let func_body = &func.block;
                    commands.push((
                        command_name.clone(),
                        func_name.clone(),
                        func_body.clone(),
                        chat_start,
                        chat_exit,
                    ));

                    if chat_start == Some(true) {
                        chat_start_cmd = Some(command_name.clone());
                    }
                    if chat_exit == Some(true) {
                        chat_exit_cmd = Some(command_name.clone());
                    }
                    if llm_req == Some(true) {
                        llm_request_cmd = Some(command_name);
                    }
                }
                new_items.push(syn::Item::Fn(func.clone()));
            }
            // Other items are pushed unchanged
            _ => new_items.push(item.clone()),
        }
    }

    let attrs_count = [&chat_start_cmd, &chat_exit_cmd, &llm_request_cmd]
        .iter()
        .filter(|&x| x.is_some())
        .count();

    if !(attrs_count == 0 || attrs_count == 3) {
        panic!("chat_start, chat_exit and llm_request need to be either all or none defined");
    }

    let handler_structs = commands.iter().map(|(command_name, _, _, _, _)| {
        let struct_name = get_cmd_struct_name(command_name);
        quote! {

            #[derive(Default)]
            struct #struct_name;

        }
    });

    let handler_impls = commands
        .iter()
        .map(|(command_name, func_name, _, chat_start, chat_exit)| {
            let struct_name = get_cmd_struct_name(command_name);
            let state = if chat_start == &Some(true) {
                quote! {
                    user.set_chat_mode(true).await;
                }
            } else if chat_exit == &Some(true) {
                quote! {
                    user.set_chat_mode(false).await;
                }
            } else {
                quote!()
            }; 
            quote! {

                #[::async_trait::async_trait]
                impl ::polybot::types::BotCommandHandler for #struct_name {
                    async fn handle(&self, user: ::polybot::types::SharedUser, args: String) -> String {
                        #state
                        #func_name(user, args).await
                    }
                }
            }
        });

    let command_insert = commands.iter().map(|(command_name, _, _, _, _)| {
        let struct_name = get_cmd_struct_name(command_name);
        quote! { handlers.insert(#command_name.to_string(), Box::new(#struct_name))}
    });

    let chat_start = match chat_start_cmd {
        Some(val) => quote! {
             fn chat_start_command() -> Option<&'static str> {
                Some(#val)
            }
        },
        None => quote! {
             fn chat_start_command() -> Option<&'static str> {
                None
            }
        },
    };
    let chat_exit = match chat_exit_cmd {
        Some(val) => quote! {
             fn chat_exit_command() -> Option<&'static str> {
                Some(#val)
            }
        },
        None => quote! {
             fn chat_exit_command() -> Option<&'static str> {
                None
            }
        },
    };

    let llm_request = match llm_request_cmd {
        Some(val) => quote! {
             fn llm_request_command() -> Option<&'static str> {
                Some(#val)
            }
        },
        None => quote! {
             fn llm_request_command() -> Option<&'static str> {
                None
            }
        },
    };

    let bot_commands_struct = quote!(
        #[derive(Default)]
        pub struct MyCommands;
    );
    let bot_commands_impl = quote! {
        impl ::polybot::types::BotCommands for MyCommands {
            fn command_list() -> ::polybot::types::CommandHashMap {
                let mut handlers: ::polybot::types::CommandHashMap = ::std::collections::HashMap::new();
                #(#command_insert;)*

                handlers
            }
            #chat_start
            #chat_exit
            #llm_request
        }
    };
    let parsed_struct: ItemStruct =
        syn::parse2(bot_commands_struct).expect("Failed to parse the MyCommands struct");

    let parsed_impl: ItemImpl = syn::parse2(bot_commands_impl)
        .expect("Failed to parse the BotCommands impl for MyCommands");

    for handler_struct in handler_structs {
        let item = handler_struct.to_string();

        let struct_str: proc_macro2::TokenStream = item
            .parse()
            .expect("Failed to convert the struct into TokenStream");

        let struct_p: ItemStruct =
            syn::parse2(struct_str).expect("problem with parsing handler struct");

        new_items.push(Item::Struct(struct_p));
    }

    for handler_impl in handler_impls {
        let item = handler_impl.to_string();

        let impl_str: proc_macro2::TokenStream = item
            .parse()
            .expect("Failed to convert the impl into TokenStream");

        let impl_p: ItemImpl = syn::parse2(impl_str).expect("problem with parsing handler struct");

        new_items.push(Item::Impl(impl_p));
    }

    // 3. Add the Parsed Items to `new_items`
    new_items.push(Item::Struct(parsed_struct));
    new_items.push(Item::Impl(parsed_impl));

    // Create new content with the original brace token and the new items
    let new_content = Some((*brace, new_items));
    let new_module = ItemMod {
        attrs: module.attrs.clone(),
        vis: module.vis.clone(),
        mod_token: module.mod_token,
        ident: module.ident.clone(),
        content: new_content,
        semi: module.semi,
    };

    let expanded = quote! {
        #new_module
    };

    TokenStream::from(expanded)
}

fn get_command_attribute(attrs: &[Attribute]) -> CommandAttribute {
    let mut cmd_attr = CommandAttribute {
        command: None,
        chat_mode_start: None,
        chat_mode_exit: None,
        llm_request: None,
    };
    for attr in attrs {
        if attr.path.is_ident(CMD_ATTR) {
            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                for nested in meta.nested {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit, path, .. })) =
                        nested
                    {
                        if path.is_ident("cmd") {
                            if let Lit::Str(lit_str) = lit {
                                cmd_attr.command = Some(lit_str.value());
                            }
                        } else if path.is_ident("chat_start") {
                            if let Lit::Bool(lit_bool) = lit {
                                cmd_attr.chat_mode_start = Some(lit_bool.value());
                            }
                        } else if path.is_ident("chat_exit") {
                            if let Lit::Bool(lit_bool) = lit {
                                cmd_attr.chat_mode_exit = Some(lit_bool.value());
                            }
                        } else if path.is_ident("llm_request") {
                            if let Lit::Bool(lit_bool) = lit {
                                cmd_attr.llm_request = Some(lit_bool.value());
                            }
                        }
                    }
                }
            }
        }
    }
    cmd_attr
}

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    //validate the attribute,
    // for now, only check that cmd is present.
    let is_cmd_in_args = args.into_iter().any(|e| {
        if let proc_macro::TokenTree::Ident(x) = e {
            x.to_string() == *"cmd"
        } else {
            false
        }
    });
    if !is_cmd_in_args {
        panic!("Handler macro used without 'cmd'!");
    }
    input
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut upper = true;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            upper = true;
        } else if upper {
            result.push(c.to_uppercase().next().unwrap());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn get_cmd_struct_name(cmd: &str) -> Ident {
    let cmd_name = to_camel_case(cmd.trim_start_matches('/'));
    Ident::new(format!("{}Handler", cmd_name).as_str(), Span::call_site())
}
