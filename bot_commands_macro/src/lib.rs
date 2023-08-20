extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Item, ItemEnum, ItemImpl, ItemMod, Lit, Meta, MetaNameValue,
    NestedMeta,
};

const CMD_ATTR: &str = "handler";

#[proc_macro_attribute]
pub fn bot_commands(_args: TokenStream, input: TokenStream) -> TokenStream {
    let module = parse_macro_input!(input as ItemMod);
    let mut commands = Vec::new();
    let mut new_items = Vec::new();

    let (brace, items) = &module.content.as_ref().unwrap();
    for item in items {
        match item {
            syn::Item::Fn(func) => {
                if let Some(command) = get_command_attribute(&func.attrs) {
                    let func_name = &func.sig.ident;
                    let command_name = command;
                    commands.push((command_name, func_name.clone()));
                }
                new_items.push(syn::Item::Fn(func.clone()));
            }
            // Other items are pushed unchanged
            _ => new_items.push(item.clone()),
        }
    }

    let variants = commands.iter().map(|(command_name, _func_name)| {
        let enum_variant_name = get_cmd_enum_variant(&command_name);
        quote! { #enum_variant_name(Command) }
    });

    let default_variant = quote! {
        DefaultCmd,
    };

    let parse_arms = commands.iter().map(|(command_name, func_name)| {
        let enum_variant_name = get_cmd_enum_variant(&command_name);
        quote! {
            #command_name => Some(Self::#enum_variant_name(Command {
                description: stringify!(#func_name).to_string(),
                handler: ::std::sync::Arc::new(|args| Box::pin(#func_name(args))),
            }))
        }
    });

    let handler_arms = commands.iter().map(|(command_name, _func_name)| {
        let enum_variant_name = get_cmd_enum_variant(&command_name);
        quote! {
            Self::#enum_variant_name(cmd) => (cmd.handler)(args)
        }
    });

    let bot_command_enum: proc_macro2::TokenStream = quote! {
        #[derive(Default)]
        pub enum BotCommand {
            #[default]
            #default_variant
            #(#variants,)*
        }
    };

    let bot_command_impl: proc_macro2::TokenStream = quote! {
        impl BotCommand {
            pub fn parse(command: &str) -> Option<Self> {
                match command {
                    #(#parse_arms,)*
                    _ => None,
                }
            }

            pub fn handler(&self, args: String) -> ::futures::future::BoxFuture<String> {
                match self {
                    Self::DefaultCmd => unimplemented!(),
                    #(#handler_arms,)*
                }
            }
        }
    };

    let command_parser_impl: proc_macro2::TokenStream = quote! {
        impl ::telegram_bot::types::CommandParser for BotCommand {
            fn parse(&self, command: &str) -> Option<Self>
            where
                Self: std::marker::Sized {
                    BotCommand::parse(command)
                }

            fn handler(&self, args: String) -> ::futures::future::BoxFuture<String> {
                self.handler(args)
            }
        }
    };
    let parsed_enum: ItemEnum =
        syn::parse2(bot_command_enum).expect("Failed to parse the generated BotCommand enum");
    let parsed_impl: ItemImpl =
        syn::parse2(bot_command_impl).expect("Failed to parse the generated impl for BotCommand");
    let parsed_cmd_parser_impl: ItemImpl = syn::parse2(command_parser_impl)
        .expect("Failed to parse the generated CommandParser impl for BotCommand");

    // 3. Add the Parsed Items to `new_items`
    new_items.push(Item::Enum(parsed_enum));
    new_items.push(Item::Impl(parsed_impl));
    new_items.push(Item::Impl(parsed_cmd_parser_impl));

    // Create new content with the original brace token and the new items
    let new_content = Some((brace.clone(), new_items));
    let new_module = ItemMod {
        attrs: module.attrs.clone(),
        vis: module.vis.clone(),
        mod_token: module.mod_token,
        ident: module.ident.clone(),
        content: new_content,
        semi: module.semi,
    };

    let expanded = quote! {
        pub struct Command {
            pub description: String,
            pub handler: ::std::sync::Arc<dyn Fn(String) -> ::futures::future::BoxFuture<'static, String> + Send + Sync>,
        }
        #new_module
    };

    TokenStream::from(expanded)
}

fn get_command_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident(CMD_ATTR) {
            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                for nested in meta.nested {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(lit_str),
                        ..
                    })) = nested
                    {
                        return Some(lit_str.value());
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("### the args are: {:#?}", args);
    //validate the attribute,
    // for now, only check that cmd is present.
    let is_cmd_in_args = args.into_iter().any(|e| {
        if let proc_macro::TokenTree::Ident(x) = e {
            x.to_string() == "cmd".to_string()
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

fn get_cmd_enum_variant(cmd: &str) -> Ident {
    let cmd_name = to_camel_case(cmd.trim_start_matches('/'));
    Ident::new(&cmd_name, Span::call_site())
}
