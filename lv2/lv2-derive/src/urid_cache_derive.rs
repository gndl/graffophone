use proc_macro::TokenStream;
use syn::{parse_macro_input, Type, Ident, Data, DataStruct, PathArguments};
use syn::DeriveInput;
use syn::Field;

struct URIDCacheField<'a> {
    struct_name: &'a Ident,
    identifier: &'a Ident,
    uridof_type: &'a Type
}

impl<'a> URIDCacheField<'a> {
    fn from_input_field(input: &'a Field, struct_name: &'a Ident) -> Self {
        Self {
            struct_name,
            identifier: input.ident.as_ref().unwrap(),
            uridof_type: &input.ty
        }
    }

    fn make_field_initialization(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            #identifier: ::lv2::urid::URIDOf::map(mapper),
        }
    }

    fn make_cache_mapping(&self) -> impl ::quote::ToTokens {
        let struct_name = self.struct_name;
        let field_name = self.identifier;
        let uridof_type = self.uridof_type;

        let inner_type = match uridof_type {
            Type::Path(path) => {
                let path_pair = path.path.segments.last().unwrap();
                match &path_pair.value().arguments {
                    PathArguments::AngleBracketed(pathargs) => {
                        let args = pathargs.args.first().unwrap();
                        (*args.value()).clone()
                    },
                    _ => unimplemented!()
                }
            },
            _ => unimplemented!()
        };

        quote! {
            impl ::lv2::urid::URIDCacheMapping<#inner_type> for #struct_name {
                fn get_urid(&self) -> #uridof_type {
                    self.#field_name
                }
            }
        }
    }
}

struct Lv2PortsFields<'a> {
    struct_name: &'a Ident,
    fields: Vec<URIDCacheField<'a>>
}

impl<'a> Lv2PortsFields<'a> {
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
            Data::Struct(DataStruct { fields, .. }) => fields.iter()
                .map(|f| URIDCacheField::from_input_field(f, struct_name))
                .collect()
        };
        Self { struct_name, fields }
    }

    fn make_derived_contents(&self) -> TokenStream {
        let struct_name = self.struct_name;

        let field_initializations = self.fields.iter().map(URIDCacheField::make_field_initialization);
        let mapping_impls = self.fields.iter().map(URIDCacheField::make_cache_mapping);

        (quote! {
            impl ::lv2::urid::URIDCache for #struct_name {
                fn new(mapper: &::lv2::urid::features::URIDMap) -> Self {
                    Self {
                        #(#field_initializations)*
                    }
                }
            }

            #(#mapping_impls)*
        }).into()
    }
}

#[inline]
pub fn urid_cache_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let list = Lv2PortsFields::from_derive_input(&input);
    list.make_derived_contents()
}
