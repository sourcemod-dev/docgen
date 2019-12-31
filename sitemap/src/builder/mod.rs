use treexml::ElementBuilder;

use mono::file::IncludeFile;
use mono::symbol::SymbolType;

pub fn build_include_tree(key: String, include: IncludeFile, base_url: &str) -> Vec<ElementBuilder> {
    let mut children: Vec<ElementBuilder> = Vec::new();

    let mut push_child = | sub_loc: &str | {
        let mut element = ElementBuilder::new("url");

        element.children(vec![
            ElementBuilder::new("loc").text(format!("{}/{}/{}", base_url, key, sub_loc)),
            ElementBuilder::new("changefreq").text("monthly"),
        ]);

        children.push(element);
    };

    macro_rules! l1 {
        ($x:expr, $t:expr) => {
            for obj in &$x {
                push_child(
                    &format_decl_name(
                        &obj.declaration.name,
                        $t,
                    ),
                );
            }
        };
    }

    push_child("");

    for m in &include.methodmaps {
        let m_name = format_decl_name(&m.declaration.name, SymbolType::MethodMap);

        push_child(&m_name);

        for func in &m.methods {
            push_child(
                &format!(
                    "{}/{}",
                    m_name,
                    format_decl_name(
                        &func.declaration.name,
                        SymbolType::Function,
                    ),
                ),
            );
        }

        for prop in &m.properties {
            push_child(
                &format!(
                    "{}/{}",
                    m_name,
                    format_decl_name(
                        &prop.declaration.name,
                        SymbolType::Property,
                    ),
                ),
            );
        }
    }

    for m in &include.enumstructs {
        let m_name = format_decl_name(&m.declaration.name, SymbolType::MethodMap);

        push_child(&m_name);

        for func in &m.methods {
            push_child(
                &format!(
                    "{}/{}",
                    m_name,
                    format_decl_name(
                        &func.declaration.name,
                        SymbolType::Function,
                    ),
                ),
            );
        }

        for f in &m.fields {
            push_child(
                &format!(
                    "{}/{}",
                    m_name,
                    format_decl_name(
                        &f.declaration.name,
                        SymbolType::Property,
                    ),
                ),
            );
        }
    }


    l1!(include.functions, SymbolType::Function);
    l1!(include.constants, SymbolType::Constant);
    l1!(include.enums, SymbolType::Enum);
    l1!(include.typesets, SymbolType::TypeSet);
    l1!(include.typedefs, SymbolType::TypeDefinition);

    children
}

fn format_decl_name(name: &str, s_type: SymbolType) -> String {
    return format!("{}.{}", s_type.to_string(), name);
}
