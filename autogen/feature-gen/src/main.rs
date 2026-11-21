use std::{
    fs::File,
    io::Write,
};

use serde::Deserialize;

use quote::quote;
use proc_macro2::{TokenStream, Span};

fn call_site() -> Span {
    Span::call_site()
}

fn ident(s: impl AsRef<str>) -> syn::Ident {
    syn::Ident::new(s.as_ref(), call_site())
}
fn path(s: impl AsRef<str>) -> syn::Path {
    syn::parse_str(s.as_ref()).unwrap()
}
struct Field {
    ident: syn::Ident,
    colon_token: syn::Token![:],
    ty: syn::Type,
}
impl syn::parse::Parse for Field {

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}
impl quote::ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}
struct IdentGenerics {
    ident: syn::Ident,
    generics: Option<syn::AngleBracketedGenericArguments>,
}

impl quote::ToTokens for IdentGenerics {

    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
    }
}

impl syn::parse::Parse for IdentGenerics {

    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            generics: input.parse().ok(),
        })
    }
}

#[derive(Deserialize)]
pub enum Fns {
    All,
    Fns(Vec<String>),
}

fn process_fns(
    base: &syn::Type,
    fns: &[String],
) -> proc_macro2::TokenStream
{
    let mut iter = fns.iter();
    let mut fn_defs = vec![];
    while iter.len() != 0 {
        let mut last = None;
        let mut function: Vec<_> = (&mut iter).take_while(|d| {
            if d.ends_with('\n') {
                true
            } else {
                last = Some(*d);
                false
            }
        }).map(|t| {
            t.trim_end_matches('\n')
        }).collect();
        function.push(last.unwrap());
        let mut iter = function.iter();
        let name = ident(iter.next().unwrap());
        let args: Vec<_> = iter
            .map(|arg| syn::parse_str::<Field>(arg)
            .unwrap())
            .collect();
        let arg_names = args
            .iter()
            .map(|arg| &arg.ident);
        fn_defs.push(quote! {
            #[inline]
            pub(crate) unsafe fn #name(&self, #(#args),*) {
                unsafe {
                    self.0.#name(#(#arg_names),*)
                }
            }
        });
    }
    quote! {
        /// An opaque handle to a the associated functions of this feature.
        pub struct Functions<'a>(#base<'a>);
        #[repr(transparent)]
        pub(crate) struct Fns<'a>(<#base<'a> as DeviceFunctionsGet<'a>>::Fn);
        impl<'a> Fns<'a> {
            #(#fn_defs)*
        }
        impl DeviceFunctions for Functions<'_> {

            type Get<'a> = Functions<'a>;
        }

        impl<'a> DeviceFunctionsGet<'a> for Functions<'a> {

            type Fn = Fns<'a>;

            fn get(self) -> sync::MappedRwLockReadGuard<'a, Self::Fn> {
                let f = self.0.get();
                sync::MappedRwLockReadGuard::map(
                    f,
                    |fns| {
                        unsafe {
                            ::core::mem::transmute::<
                                &<#base<'a> as DeviceFunctionsGet<'a>>::Fn,
                                &Fns,
                            >(fns)
                        }
                    }
                )
            }
        }
    }
}

#[derive(Deserialize)]
struct ExtensionFunctions {
    base: String,
    get_fns: String,
    fns: Fns,
}

#[derive(Deserialize)]
struct Extension {
    path: Option<String>,
    functions: Option<ExtensionFunctions>,
    no_core_feature: Option<bool>,
    features: Option<String>,
    vk_properties: Option<String>,
    property_docs: Option<Vec<String>>,
    property_fields: Option<Vec<String>>,
    property_mappings: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Feature {
    title: String,
    doc: Vec<String>,
    alias: Option<String>,
    extension: Option<Extension>,
    enums: Option<Vec<String>>,
    flags32: Option<Vec<String>>,
}

impl Feature {

    fn create_module(
        &self,
        module: &str,
        min_version: Option<&syn::Ident>,
        get_features: Option<&IdentGenerics>,
        set_features: Option<&IdentGenerics>,
    ) -> TokenStream {
        let title = ident(&self.title);
        let docs = self.doc
            .iter()
            .map(|doc| quote! {
                #[doc = #doc]
            });
        let module_path = path(format!("{module}::{title}"));
        let enums: Box<[_]> = self.enums
            .iter()
            .flatten()
            .map(|e| {
                path(e)
            }).collect();
        let enums = (!enums.is_empty()).then(|| quote! {
            ctx.add_supported_enums([
                #(#enums),*
            ]);
        });
        let flags32: Box<[_]> = self.flags32
            .iter()
            .flatten()
            .map(|flag| {
                let path = path(flag);
                quote! {
                    ctx.add_supported_flags32(
                        #path
                    );
                }
            }).collect();
        let alias = self.alias.as_ref().map(ident);
        let vk_feature = alias.unwrap_or(title.clone());
        let mut add_required_extension = None;
        let mut add_optional_extension = None;
        let mut add_optional_extension_alt = None;
        let ext = self.extension.as_ref().map(|ext| {
            if let Some(p) = &ext.path {
                let p = path(p);
                add_required_extension = Some(quote! {
                    ctx.add_required_extension(#p)?;
                });
                add_optional_extension = Some(quote! {
                    if !ctx.add_optional_extension(#p) {
                        log::warn!("`{}` not supported",
                            stringify!(#module_path)
                        );
                        return Ok(());
                    }
                });
                add_optional_extension_alt = Some(quote! {
                    && !ctx.add_optional_extension(#p)
                });
            }
            ext.features
                .as_ref()
                .map(|s| syn::parse_str::<syn::Path>(s).unwrap())
        });
        let (require_supported, enable_supported) = if let Some(ext) = &ext {
            let ext_features = &ext;
            if let Some(min_version) = min_version {
                if let Some(ext_features) = ext_features {
                    if self.extension.as_ref().unwrap().no_core_feature
                        .is_some_and(|b| b)
                    {
                        (
                        quote! {
                            if ctx.api_version() < #min_version {
                                #add_required_extension
                                let features = ctx.get_features::<#ext_features>();
                                if features.#vk_feature == 0 {
                                    Err(err!(
                                        "`{}` not supported",
                                        stringify!(#module_path),
                                    ))?
                                }
                            }
                        },
                        quote! {
                            if ctx.api_version() < #min_version {
                                #add_optional_extension
                                let features = ctx.get_features::<#ext_features>();
                                if features.#vk_feature == 0 {
                                    log::warn!("`{}` not supported",
                                        stringify!(#module_path)
                                    )
                                } else {
                                    enable.store(true, atomic::Ordering::Release);
                                }
                            } else {
                                enable.store(true, atomic::Ordering::Release)
                            }
                        }
                        )
                    } else {
                        (
                            quote! {
                                if ctx.api_version() < #min_version {
                                    #add_required_extension
                                    let features = ctx.get_features::<#ext_features>();
                                    if features.#vk_feature == 0 {
                                        Err(err!(
                                            "`{}` not supported",
                                            stringify!(#module_path),
                                        ))?
                                    }
                                } else {
                                    let features = ctx.#get_features();
                                    if features.#vk_feature == 0 {
                                        Err(err!(
                                            "`{}` not supported",
                                            stringify!(#module_path),
                                        ))?
                                    }
                                }
                            },
                            quote! {
                                if ctx.api_version() < #min_version {
                                    #add_optional_extension
                                    let features = ctx.get_features::<#ext_features>();
                                    if features.#vk_feature == 0 {
                                        log::warn!("`{}` not supported",
                                            stringify!(#module_path)
                                        )
                                    } else {
                                        enable.store(true, atomic::Ordering::Release);
                                    }
                                } else {
                                    let features = ctx.#get_features();
                                    if features.#vk_feature == 0 {
                                        log::warn!("`{}` not supported",
                                            stringify!(#module_path)
                                        )
                                    } else {
                                        enable.store(true, atomic::Ordering::Release);
                                    }
                                }
                            }
                        )
                    }
                } else if self.extension.as_ref().unwrap().no_core_feature
                    .is_some_and(|b| b)
                {
                    (
                        quote! {
                            if ctx.api_version() < #min_version {
                                #add_required_extension
                            }
                        },
                        quote! {
                            if ctx.api_version() < #min_version
                                #add_optional_extension_alt
                            {
                                log::warn!("`{}` not supported",
                                    stringify!(#module_path)
                                )
                            } else {
                                enable.store(true, atomic::Ordering::Release);
                            }
                        }
                    )
                } else {
                    (
                        quote! {
                            if ctx.api_version() < #min_version {
                                #add_required_extension
                            } else {
                                let features = ctx.#get_features();
                                if features.#vk_feature == 0 {
                                    Err(err!(
                                        "`{}` not supported",
                                        stringify!(#module_path),
                                    ))?
                                }
                            }
                        },
                        quote! {
                            if ctx.api_version() < #min_version {
                                #add_optional_extension
                                enable.store(true, atomic::Ordering::Release);
                            } else {
                                let features = ctx.#get_features();
                                if features.#vk_feature == 0 {
                                    log::warn!("`{}` not supported",
                                        stringify!(#module_path)
                                    )
                                } else {
                                    enable.store(true, atomic::Ordering::Release);
                                }
                            }
                        }
                    )
                }
            } else if let Some(ext_features) = ext_features {
                (
                    quote! {
                        #add_required_extension
                        let features = ctx.get_features::<#ext_features>();
                        if features.#vk_feature == 0 {
                            Err(err!(
                                "`{}` not supported",
                                stringify!(#module_path),
                            ))?
                        }
                    },
                    quote! {
                        #add_optional_extension
                        let features = ctx.get_features::<#ext_features>();
                        if features.#vk_feature == 0 {
                            log::warn!("`{}` not supported",
                                stringify!(#module_path)
                            )
                        } else {
                            enable.store(true, atomic::Ordering::Release);
                        }
                    },
                )
                
            } else {
                (
                    quote! {
                        #add_required_extension
                    },
                    quote! {
                        #add_optional_extension
                        enable.store(true, atomic::Ordering::Release);
                    },
                )
            }
        } else {
            (
                quote! {
                    let features = ctx.#get_features();
                    if features.#vk_feature == 0 {
                        Err(err!(
                            "`{}` not supported",
                            stringify!(#module_path),
                        ))?
                    }
                },
                quote! {
                    let features = ctx.#get_features();
                    if features.#vk_feature == 0 {
                        log::warn!("`{}` not supported",
                            stringify!(#module_path)
                        )
                    } else {
                        enable.store(true, atomic::Ordering::Release);
                    }
                }
            )
        };
        let enable_feature = if let Some(min_version) = min_version 
        {
            if let Some(ext_features) = &ext {
                if self.extension.as_ref().unwrap().no_core_feature
                    .is_some_and(|b| b)
                {
                    quote! {}
                } else if let Some(ext_features) = &ext_features {
                    quote! {
                        if ctx.api_version() < #min_version {
                            ctx.set_features::<#ext_features, _>(|features| {
                                features.#vk_feature(true)
                            });
                        } else {
                            ctx.#set_features(|features| {
                                features.#vk_feature(true)
                            });
                        }
                    }
                } else {
                    quote! {
                        if ctx.api_version() >= #min_version {
                            ctx.#set_features(|features| {
                                features.#vk_feature(true)
                            });
                        }
                    }
                }
            } else {
                quote! {
                    ctx.#set_features(|features| {
                        features.#vk_feature(true)
                    });
                }
            }
        } else {
            let ext_features = ext.as_ref().unwrap();
            if ext_features.is_some() {
                quote! {
                    ctx.set_features::<#ext_features, _>(|features| {
                        features.#vk_feature(true)
                    });
                }
            } else {
                quote! {}
            }
        };
        let mut properties = None;
        let mut property_data_void = false;
        if let Some(ext) = &self.extension &&
            let Some(fields) = &ext.property_fields
        { 
            let property_mappings = ext.property_mappings.as_ref().unwrap();
            let mut property_docs = ext.property_docs.as_ref().unwrap().iter();
            let mut field_defs = Vec::with_capacity(fields.len());
            let mut field_mappings = Vec::with_capacity(fields.len());
            for (i, field) in fields.iter().enumerate() {
                let mapping = ident(&property_mappings[i]);
                let field: Field = syn::parse_str(field).unwrap();
                let mut last = None;
                let mut docs: Vec<_> = (&mut property_docs)
                    .take_while(|d| {
                        if d.ends_with('\n') {
                            true
                        } else {
                            last = Some(*d);
                            false
                        }
                    })
                    .map(|t| {
                        let d = t.trim_end_matches('\n');
                        quote! {
                            #[doc = #d]
                        }
                    })
                    .collect();
                let last = last.unwrap();
                docs.push(quote! { #[doc = #last] });
                field_defs.push(quote! {
                    #(#docs)*
                    pub #field,
                });
                let name = &field.ident;
                if let syn::Type::Path(p) = &field.ty &&
                    p.path.is_ident("bool")
                {
                    field_mappings.push(quote! {
                        #name: value.#mapping != 0,
                    });
                } else {
                    field_mappings.push(quote! {
                        #name: value.#mapping,
                    });
                }
            }
            let vk_properties = path(ext.vk_properties.as_ref().unwrap());
            properties = Some((quote! {
                /// Properties of this feature.
                pub struct Properties {
                    #(#field_defs)*
                }

                impl From<#vk_properties<'static>> for Properties {
                    #[inline]
                    fn from(value: #vk_properties<'static>) -> Self {
                        Self {#(
                            #field_mappings
                        )*}
                    }
                }
            }, vk_properties));
        }
        let mut prop_def = None;
        let mut fn_def = None;
        let (feature_data, create_feature_data) = if let Some((def, prop)) = &properties
        {
            prop_def = Some(def);
            if let Some(functions) = self.extension
                .as_ref()
                .and_then(|ext| ext.functions.as_ref())
            {
                let base: syn::Type = syn::parse_str(&functions.base).unwrap();
                let get_fns: syn::Ident = syn::parse_str(&functions.get_fns).unwrap();
                let (make_fns, name_fns) = match &functions.fns {
                    Fns::All => (quote! {
                        ExtensionFunctions::<#base>(#get_fns as fn(&Device) -> #base),
                    }, quote! { ExtensionFunctions<#base> }),
                    Fns::Fns(fns) => {
                        fn_def = Some(process_fns(&base, fns));
                        (quote! {
                            ExtensionFunctions::<Functions>(|device| Functions(#get_fns(device)))
                        }, quote! { ExtensionFunctions<Functions> })
                    },
                };
                (
                    quote! {
                        (Properties, #name_fns)
                    },
                    quote! {
                        let mut properties = #prop::default();
                        ctx.get_physical_device_properties(&mut properties);
                        Some((
                            Properties::from(properties),
                            #make_fns,
                        ))
                    }
                )
            } else {
                (
                    quote! { Properties },
                    quote! {
                        let mut properties = #prop::default();
                        ctx.get_physical_device_properties(&mut properties);
                        Some(
                            Properties::from(properties),
                        )
                    },
                )
            }
        } else if let Some(functions) = self.extension
            .as_ref()
            .and_then(|ext| ext.functions.as_ref())
        {
            let base: syn::Type = syn::parse_str(&functions.base).unwrap();
            let get_fns: syn::Ident = syn::parse_str(&functions.get_fns).unwrap();
            let (make_fns, name_fns) = match &functions.fns {
                Fns::All => (quote! {
                    ExtensionFunctions::<#base>(#get_fns as fn(&Device) -> #base),
                }, quote! { ExtensionFunctions<#base> }),
                Fns::Fns(fns) => {
                    fn_def = Some(process_fns(&base, fns));
                    (quote! {
                        ExtensionFunctions::<Functions>(|device| Functions(#get_fns(device)))
                    }, quote! { ExtensionFunctions<Functions> })
                },
            };
            (
                quote! {
                    #name_fns
                },
                quote! {
                    Some(
                        #make_fns
                    )
                }
            )
        } else {
            property_data_void = true;
            (quote! {()}, quote! { Some(()) })
        };
        let (enable_require, enable_enable) = if enable_feature.is_empty() &&
            property_data_void
        {
            (quote! {
                |_| { Some(()) }
            }, quote! { move |_| supported_enable
                .load(atomic::Ordering::Acquire)
                .then_some(())
            })
        } else {
            let ctx = if enable_feature.is_empty() && enums.is_none() {
                quote! { _ }
            } else {
                quote! { ctx }
            };
            (quote! {
                |#ctx| {
                    #enable_feature
                    #enums
                    #(#flags32)*
                    #create_feature_data
                }
            }, quote! { move |#ctx| {
                    if !supported_enable.load(atomic::Ordering::Acquire) {
                        return None;
                    }
                    #enable_feature
                    #enums
                    #(#flags32)*
                    #create_feature_data
                }
            })
        };
        quote! {
            #(#docs)*
            pub mod #title {

                use super::*;
 
                #prop_def

                #fn_def

                /// The name of this feature.
                pub static NAME: FeatureName<#feature_data> = FeatureName::new(stringify!(#title));

                /// Enables this feature and requires for it to be supported.
                pub fn require() -> DeviceFeature {
                    feature(
                        |ctx| {
                            #require_supported
                            Ok(())
                        },
                        #enable_require,
                        NAME,
                    )
                }

                /// Tries to enable this feature, but will only warn if it is not supported.
                pub fn enable() -> DeviceFeature {
                    use crate::sync::{Arc, atomic::{self, AtomicBool}};
                    let enable = Arc::new(AtomicBool::new(false));
                    let supported_enable = enable.clone();
                    feature(
                        move |ctx| {
                            #enable_supported
                            Ok(())
                        },
                        #enable_enable,
                        NAME,
                    )
                } 
            }
        }
    }
}

#[derive(Deserialize)]
struct Module {
    doc: Vec<String>,
    version: Option<String>,
    get_features: Option<String>,
    set_features: Option<String>,
    features: Vec<Feature>,
}

#[derive(Deserialize)]
pub struct Document {
    vk_10: Module,
    vk_11: Module,
    vk_12: Module,
    vk_13: Module,
    vk_14: Module,
    vk_khr: Module,
    vk_ext: Module,
}

impl Document {

    fn process_module(
        module: &Module,
        name: &str,
        file: &mut File,
    ) {
        let docs = module.doc
            .iter()
            .map(|doc| quote! {
                #[doc = #doc]
            });
        let get_features = module.get_features
            .as_ref()
            .map(|s| syn::parse_str(s).unwrap());
        let set_features = module.set_features
            .as_ref()
            .map(|s| syn::parse_str(s).unwrap());
        let version = module.version.as_ref()
            .map(ident);
        let sub_modules = module.features
            .iter()
            .map(|feature| {
                feature.create_module(name, version.as_ref(), get_features.as_ref(), set_features.as_ref())
            });
        let name = ident(name);
        let module = quote! {
            #(#docs)*
            pub mod #name {

                use crate::features::*;
                
                #(#sub_modules)*
            }
        };
        write!(file, "{module}").unwrap();
    }

    fn process(
        &self,
        file: &mut File,
    ) {
        Self::process_module(&self.vk_10, "vk_10", file);
        Self::process_module(&self.vk_11, "vk_11", file);
        Self::process_module(&self.vk_12, "vk_12", file);
        Self::process_module(&self.vk_13, "vk_13", file);
        Self::process_module(&self.vk_14, "vk_14", file);
        Self::process_module(&self.vk_khr, "vk_khr", file);
        Self::process_module(&self.vk_ext, "vk_ext", file);
    } 
}

fn main() -> Result<(), toml::de::Error> {
    let features: Document = toml::from_str(include_str!("../Features.toml"))?;
    let mut file = File::create("features.rs").unwrap();
    write!(file, "{}",  quote!{
        //! This file is autogenerated, do not modify manually.
    }).unwrap();
    features.process(&mut file);
    std::process::Command
        ::new("rustfmt")
        .arg("features.rs")
        .output()
        .unwrap();
    std::process::Command
        ::new("mv")
        .arg("features.rs")
        .arg("../../src/features/generated.rs")
        .output()
        .unwrap();
    Ok(())
}
