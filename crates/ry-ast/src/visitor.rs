use ry_proc_macros::visit_fn;
use string_interner::DefaultSymbol;

use crate::*;

pub trait Visitor: Sized {
    visit_fn!(program_unit for &ProgramUnit {
        for import in &node.imports {
            self.visit_import(import);
        }

        for item in &node.items {
            self.visit_item((&item.0, &item.1));
        }
    });

    visit_fn!(import for &Import);
    visit_fn!(import_after_first_item for &Import);

    visit_fn!(items for &Items {
        for item in node {
            self.visit_item((&item.0, &item.1));
        }
    });
    visit_fn!(item for (&str, &Item) {
        match node.1 {
            Item::EnumDecl(e) => self.visit_enum_decl((node.0, e)),
            Item::FunctionDecl(f) => self.visit_function_decl((node.0, f)),
            Item::StructDecl(s) => self.visit_struct_decl((node.0, s)),
            Item::Impl(i) => self.visit_impl((node.0, i)),
            Item::TraitDecl(t) => self.visit_trait_decl((node.0, t)),
            Item::Import(i) => self.visit_import_after_first_item(i),
        }
    });

    visit_fn!(enum_decl for (&str, &EnumDecl));
    visit_fn!(function_decl for (&str, &FunctionDecl));
    visit_fn!(struct_decl for (&str, &StructDecl));
    visit_fn!(trait_decl for (&str, &TraitDecl));
    visit_fn!(r#impl for (&str, &Impl));

    visit_fn!(generic_annotations for &GenericAnnotations);
    visit_fn!(generic_annotation for &GenericAnnotation);

    visit_fn!(generics for &Vec<Type> {
        for generic in node {
            self.visit_generic(generic);
        }
    });
    visit_fn!(generic for &Type {
        self.visit_type(node);
    });

    visit_fn!(r#type for &Type {
        match &*node.value {
            RawType::Array(inner) => self.visit_array(inner),
            RawType::NegativeTrait(r#trait) => self.visit_negative_trait(r#trait),
            RawType::Option(inner) => self.visit_option(inner),
            RawType::Primary(name, generics) => self.visit_primary((name, generics)),
            RawType::Reference(mutable, inner) => self.visit_reference((*mutable, inner)),
        }
    });

    visit_fn!(array for &Type);
    visit_fn!(option for &Type);
    visit_fn!(reference for (bool, &Type));
    visit_fn!(primary for (&WithSpan<Vec<DefaultSymbol>>, &Vec<Type>) {
        self.visit_static_name(node.0);

        for generic in node.1 {
            self.visit_type(generic);
        }
    });
    visit_fn!(negative_trait for &Type);

    visit_fn!(expr for &Expression);

    visit_fn!(bool_literal for bool);
    visit_fn!(integer_literal for u64);
    visit_fn!(float_literal for f64);
    visit_fn!(imaginary_literal for f64);
    visit_fn!(string_literal for &str);

    visit_fn!(binary_expr for (&Expression, &Token, &Expression));

    visit_fn!(op for &Token);
    visit_fn!(static_name for &WithSpan<Vec<DefaultSymbol>>);
}
