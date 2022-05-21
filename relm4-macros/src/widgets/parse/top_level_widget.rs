use syn::parse::ParseStream;

use crate::widgets::{
    parse_util, Attr, Attrs, Properties, Property, PropertyName, PropertyType, TopLevelWidget,
    Widget, WidgetAttr, WidgetFunc,
};

impl TopLevelWidget {
    pub(super) fn parse(input: ParseStream) -> Self {
        let attributes: Option<Attrs> = input.parse().ok();

        // Look for #[root] attribute and remove it from the list if it exists
        let (attributes, root_attr) = if let Some(prev_attributes) = attributes {
            let mut attributes = Attrs {
                inner: Vec::with_capacity(prev_attributes.inner.len()),
            };
            let mut root_attr = None;
            for attr in prev_attributes.inner.into_iter() {
                match attr {
                    Attr::Root(ident) => {
                        // Save root attribute and don't push it to the new list
                        root_attr = Some(ident);
                    }
                    _ => attributes.inner.push(attr),
                }
            }
            (Some(attributes), root_attr)
        } else {
            (None, None)
        };

        let inner = match Widget::parse(input, attributes, None) {
            Ok(inner) => inner,
            Err(err) => Widget {
                doc_attr: None,
                attr: WidgetAttr::None,
                mutable: None,
                name: parse_util::string_to_snake_case("incorrect_top_level_widget"),
                func: WidgetFunc {
                    path: parse_util::strings_to_path(&["gtk", "Box"]),
                    args: None,
                    method_chain: None,
                    ty: None,
                },
                args: None,
                properties: Properties {
                    properties: vec![Property {
                        name: PropertyName::Ident(parse_util::string_to_snake_case(
                            "invalid_property",
                        )),
                        ty: PropertyType::ParseError(err),
                    }],
                },
                wrapper: None,
                ref_token: None,
                deref_token: None,
                returned_widget: None,
            },
        };

        Self { root_attr, inner }
    }
}
