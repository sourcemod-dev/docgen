//! Walks the tree-sitter syntax tree and extracts documented symbols.

use std::collections::HashMap;

use tree_sitter::Node;

use schema::bundle::Strand;
use schema::symbol::{
    parse_type_signature, Argument, Constant, Declaration, Define, DocLocation, Documentation,
    EnumStruct, Entry, Enumeration, Field, Function, MethodMap, Property, Type, TypeDefinition,
    TypeSet,
};

use crate::comments::Comments;
use crate::expr::{old_tag_to_type, Renderer};
use crate::preprocess::Preprocessed;

pub struct Extractor<'a> {
    /// Original source, used for comment text and define values
    pub source: &'a str,
    /// Preprocessed source that was handed to tree-sitter
    pub text: &'a str,
    pub pp: &'a Preprocessed,
    pub comments: Comments,
    pub line_starts: Vec<usize>,
    pub strand: Strand,
}

fn line_of(node: &Node) -> usize {
    node.start_position().row + 1
}

fn children<'t>(node: &Node<'t>) -> Vec<Node<'t>> {
    let mut c = node.walk();
    node.children(&mut c).collect()
}

/// Children along with the field name they're assigned to
fn children_with_fields<'t>(node: &Node<'t>) -> Vec<(Option<&'static str>, Node<'t>)> {
    let mut out = Vec::new();
    let mut c = node.walk();
    if c.goto_first_child() {
        loop {
            out.push((c.field_name(), c.node()));
            if !c.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

fn field_nodes<'t>(node: &Node<'t>, name: &str) -> Vec<Node<'t>> {
    let mut c = node.walk();
    node.children_by_field_name(name, &mut c).collect()
}

/// Structured type of a parameter, field or return value
#[derive(Default, Debug)]
struct TypeParts {
    is_const: bool,
    base: String,
    prefix_dims: Vec<String>,
    postfix_dims: Vec<String>,
    by_ref: bool,
}

impl TypeParts {
    fn dims(&self) -> String {
        let mut s = String::new();
        for d in self.prefix_dims.iter().chain(self.postfix_dims.iter()) {
            s += d;
        }
        s
    }

    /// `const char[]`, `int&`, `float[3]`
    fn type_string(&self) -> String {
        let mut s = String::new();
        if self.is_const {
            s += "const ";
        }
        s += &self.base;
        s += &self.dims();
        if self.by_ref {
            s += "&";
        }
        s
    }

    /// `const char[] name`, `float vec[3]`, `int& value`
    fn decl_string(&self, name: &str) -> String {
        let mut s = String::new();
        if self.is_const {
            s += "const ";
        }
        s += &self.base;
        if self.postfix_dims.is_empty() {
            for d in &self.prefix_dims {
                s += d;
            }
            if self.by_ref {
                s += "&";
            }
            s += " ";
            s += name;
        } else {
            for d in &self.prefix_dims {
                s += d;
            }
            if self.by_ref {
                s += "&";
            }
            s += " ";
            s += name;
            for d in &self.postfix_dims {
                s += d;
            }
        }
        s
    }
}

impl<'a> Extractor<'a> {
    pub fn new(source: &'a str, pp: &'a Preprocessed) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Extractor {
            source,
            text: &pp.text,
            pp,
            comments: Comments::new(&pp.items),
            line_starts,
            strand: Strand::default(),
        }
    }

    fn line_of_offset(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }

    fn text(&self, node: &Node) -> &'a str {
        &self.text[node.start_byte()..node.end_byte()]
    }

    fn documentation(&self, line: usize) -> Documentation {
        match self.comments.find(line) {
            Some(r) => Documentation {
                ref_line: line as u64,
                doc_start: DocLocation::from(r.start),
                doc_end: DocLocation::from(r.end),
                docs: Some(spdcp::Comment::parse(&self.source[r.start..r.end])),
                metadata: None,
            },
            None => Documentation {
                ref_line: line as u64,
                doc_start: DocLocation::default(),
                doc_end: DocLocation::default(),
                docs: None,
                metadata: None,
            },
        }
    }

    fn declaration(&self, name: &str, line: usize) -> Declaration {
        Declaration {
            name: name.to_string(),
            documentation: self.documentation(line),
        }
    }

    fn render_expr(&self, node: &Node) -> String {
        Renderer::new(self.pp, node.start_byte()).render_src(self.text(node))
    }

    fn dim(&self, node: &Node) -> String {
        match node.kind() {
            "fixed_dimension" => {
                // `[` expr (char)? `]`
                let expr = children(node)
                    .into_iter()
                    .find(|c| c.is_named() && c.kind() != "dimension_packing" && c.kind() != "comment");
                match expr {
                    // Like docparse, only simple sizes are shown
                    Some(e) => match self.render_expr(&e) {
                        size if size.contains(' ') => "[]".to_string(),
                        size => format!("[{}]", size),
                    },
                    None => "[]".to_string(),
                }
            }
            _ => "[]".to_string(),
        }
    }

    /// Base type name from a `type`, `old_type` or `builtin_type` node
    fn base_type(&self, node: &Node) -> String {
        match node.kind() {
            "old_type" => {
                let inner = children(node).into_iter().find(|c| c.kind() != ":");
                match inner {
                    Some(n) if n.kind() == "multi_tag" => "any".to_string(),
                    Some(n) => old_tag_to_type(self.text(&n).trim()).to_string(),
                    None => "int".to_string(),
                }
            }
            "array_type" => children(node)
                .into_iter()
                .find(|c| c.kind() == "type")
                .map(|t| self.base_type(&t))
                .unwrap_or_default(),
            _ => match self.text(node).trim() {
                // Untagged old style parameters like `...`
                "" => "int".to_string(),
                t => t.to_string(),
            },
        }
    }

    /// Collects type information from a node's `type`/`returnType` field
    /// and any dimensions written after it.
    fn type_parts(&self, nodes: &[Node]) -> TypeParts {
        let mut parts = TypeParts {
            base: "int".to_string(),
            ..Default::default()
        };
        for n in nodes {
            match n.kind() {
                "type" | "old_type" | "builtin_type" => parts.base = self.base_type(n),
                "array_type" => {
                    parts.base = self.base_type(n);
                    for c in children(n) {
                        if matches!(c.kind(), "dimension" | "fixed_dimension") {
                            parts.prefix_dims.push(self.dim(&c));
                        }
                    }
                }
                "dimension" | "fixed_dimension" => parts.prefix_dims.push(self.dim(n)),
                "&" => parts.by_ref = true,
                _ => {}
            }
        }
        parts
    }

    fn parameter(&self, node: &Node) -> Option<Argument> {
        match node.kind() {
            "parameter_declaration" => {
                let mut parts = self.type_parts(&field_nodes(node, "type"));
                let mut name = String::new();
                let mut default = None;
                let mut seen_name = false;
                for (field, c) in children_with_fields(node) {
                    match c.kind() {
                        "variable_storage_class" => parts.is_const = true,
                        "&" => parts.by_ref = true,
                        "dimension" | "fixed_dimension" if seen_name => {
                            parts.postfix_dims.push(self.dim(&c))
                        }
                        _ => {}
                    }
                    if field == Some("name") {
                        name = self.text(&c).to_string();
                        seen_name = true;
                    }
                }
                if let Some(d) = node.child_by_field_name("defaultValue") {
                    default = Some(self.render_expr(&d));
                }
                // Old style `String:name[]` is printed as `char[] name`, sized arrays keep
                // their dimensions after the name
                let old_style = field_nodes(node, "type").is_empty()
                    || field_nodes(node, "type").iter().any(|t| t.kind() == "old_type");
                if old_style && parts.postfix_dims.iter().all(|d| d == "[]") {
                    let postfix = std::mem::take(&mut parts.postfix_dims);
                    parts.prefix_dims.extend(postfix);
                }
                Some(Argument {
                    r#type: parts.type_string(),
                    decl: parts.decl_string(&name),
                    name,
                    default,
                })
            }
            "rest_parameter" => {
                let mut parts = self.type_parts(&field_nodes(node, "type"));
                for c in children(node) {
                    if c.kind() == "variable_storage_class" {
                        parts.is_const = true;
                    }
                }
                let ty = format!("{}...", parts.type_string());
                Some(Argument {
                    r#type: ty.clone(),
                    name: "...".to_string(),
                    decl: ty,
                    default: None,
                })
            }
            _ => None,
        }
    }

    fn parameters(&self, node: Option<Node>) -> Vec<Argument> {
        match node {
            Some(n) => children(&n)
                .iter()
                .filter_map(|c| self.parameter(c))
                .collect(),
            None => Vec::new(),
        }
    }

    fn return_type(&self, node: &Node) -> String {
        let nodes = field_nodes(node, "returnType");
        if nodes.is_empty() {
            // Old style declarations without a tag. docparse inferred the
            // return type of definitions from their body.
            let returns_value = node.kind() == "function_definition"
                && node
                    .child_by_field_name("body")
                    .map_or(false, |b| returns_value(&self.source[b.start_byte()..b.end_byte()]));
            return match node.kind() {
                "function_definition" if !returns_value => "void",
                _ => "int",
            }
            .to_string();
        }
        self.type_parts(&nodes).type_string()
    }

    fn name(&self, node: &Node) -> Option<String> {
        field_nodes(node, "name")
            .into_iter()
            .find(|n| n.kind() == "identifier")
            .map(|n| self.text(&n).to_string())
    }

    fn name_line(&self, node: &Node) -> usize {
        field_nodes(node, "name")
            .first()
            .map(line_of)
            .unwrap_or_else(|| line_of(node))
    }

    /// `function void(int client)`
    fn signature(&self, return_type: &str, params: &[Argument]) -> String {
        let args: Vec<&str> = params.iter().map(|a| a.decl.as_str()).collect();
        format!("function {}({})", return_type, args.join(", "))
    }

    pub fn run(&mut self, root: &Node) {
        for node in children(root) {
            match node.kind() {
                "function_declaration" | "function_definition" => self.function(&node),
                "enum" => self.enumeration(&node),
                "enum_struct" => self.enum_struct(&node),
                "methodmap" => self.methodmap(&node),
                "typedef" => self.typedef(&node),
                "typeset" => self.typeset(&node),
                "functag" => self.functag(&node),
                "funcenum" => self.funcenum(&node),
                "struct" => self.structure(&node),
                _ => {}
            }
        }

        self.defines();
        self.space_foreign_rest_types();
    }

    /// docparse printed rest parameters of types it couldn't resolve (types
    /// declared in other files) as `Type ...` and all others as `Type...`
    fn space_foreign_rest_types(&mut self) {
        const BUILTIN: &[&str] = &["int", "bool", "float", "char", "any", "void", "int64"];
        let s = &mut self.strand;
        let local: std::collections::HashSet<String> = s
            .enums
            .keys()
            .chain(s.methodmaps.keys())
            .chain(s.enumstructs.keys())
            .chain(s.typedefs.keys())
            .chain(s.typesets.keys())
            .cloned()
            .collect();

        let is_local = |t: &str| {
            let name = t
                .trim_start_matches("const ")
                .split(|c| c == '[' || c == '&' || c == ' ' || c == '.')
                .next()
                .unwrap_or("");
            BUILTIN.contains(&name) || local.contains(name)
        };

        let fix = |f: &mut Function| {
            // Unresolved by-ref types were printed as `Handle &name`
            for a in f.arguments.iter_mut() {
                if a.name != "..." && a.r#type.ends_with('&') && !is_local(&a.r#type) {
                    a.decl = format!("{} &{}", a.r#type.trim_end_matches('&'), a.name);
                }
            }
            for a in f.arguments.iter_mut().filter(|a| a.name == "...") {
                let base = a.r#type.trim_end_matches("...").trim_end();
                let t = if is_local(base) {
                    format!("{}...", base)
                } else {
                    format!("{} ...", base)
                };
                a.r#type = t.clone();
                a.decl = t;
            }
        };

        s.functions.values_mut().for_each(fix);
        s.methodmaps
            .values_mut()
            .flat_map(|m| m.methods.values_mut())
            .for_each(fix);
        s.enumstructs
            .values_mut()
            .flat_map(|m| m.methods.values_mut())
            .for_each(fix);
    }

    /// Legacy `struct`s such as `Plugin` and `Extension` are documented as enum structs
    fn structure(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let mut es = EnumStruct {
            declaration: self.declaration(&name, line_of(node)),
            methods: HashMap::new(),
            fields: HashMap::new(),
        };
        for member in children(node) {
            if !matches!(member.kind(), "struct_field" | "old_struct_field") {
                continue;
            }
            let fname = match self.name(&member) {
                Some(n) => n,
                None => continue,
            };
            let mut parts = self.type_parts(&field_nodes(&member, "type"));
            let mut seen_name = false;
            for (field, c) in children_with_fields(&member) {
                match c.kind() {
                    "const" => parts.is_const = true,
                    "dimension" | "fixed_dimension" if seen_name => parts.postfix_dims.push(self.dim(&c)),
                    _ => {}
                }
                if field == Some("name") {
                    seen_name = true;
                }
            }
            es.fields.insert(
                fname.clone(),
                Field {
                    declaration: self.declaration(&fname, line_of(&member)),
                    r#type: parts.type_string(),
                },
            );
        }
        self.strand.enumstructs.insert(name, es);
    }

    fn function_kind(&self, node: &Node) -> String {
        if let Some(k) = node.child_by_field_name("kind") {
            return self.text(&k).to_string();
        }
        match node.child_by_field_name("visibility") {
            Some(v) => {
                // Normalized order, `stock static` is `static stock`
                let words: Vec<&str> = self.text(&v).split_whitespace().collect();
                ["public", "static", "stock"]
                    .iter()
                    .filter(|w| words.contains(w))
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            None => "function".to_string(),
        }
    }

    fn function(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let f = Function {
            declaration: self.declaration(&name, self.name_line(node)),
            kind: self.function_kind(node),
            return_type: self.return_type(node),
            arguments: self.parameters(node.child_by_field_name("parameters")),
        };
        self.strand.functions.insert(name, f);
    }

    fn enumeration(&mut self, node: &Node) {
        let mut entries = Vec::new();
        if let Some(list) = node.child_by_field_name("entries") {
            for e in children(&list) {
                if e.kind() != "enum_entry" {
                    continue;
                }
                let name = match self.name(&e) {
                    Some(n) => n,
                    None => continue,
                };
                let value = field_nodes(&e, "value")
                    .into_iter()
                    .filter(|v| v.is_named())
                    .last()
                    .map(|v| self.render_expr(&v));
                let line = self.name_line(&e);
                entries.push(Entry {
                    declaration: self.declaration(&name, line),
                    value,
                });
            }
        }

        match self.name(node) {
            None => {
                for e in entries {
                    self.strand.constants.insert(
                        e.declaration.name.clone(),
                        Constant {
                            declaration: e.declaration,
                        },
                    );
                }
            }
            Some(name) => {
                let mut map = HashMap::new();
                for e in entries {
                    map.insert(e.declaration.name.clone(), e);
                }
                self.strand.enums.insert(
                    name.clone(),
                    Enumeration {
                        declaration: self.declaration(&name, line_of(node)),
                        entries: map,
                    },
                );
            }
        }
    }

    fn enum_struct(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let mut es = EnumStruct {
            declaration: self.declaration(&name, line_of(node)),
            methods: HashMap::new(),
            fields: HashMap::new(),
        };
        for member in children(node) {
            match member.kind() {
                "enum_struct_field" => {
                    let fname = match self.name(&member) {
                        Some(n) => n,
                        None => continue,
                    };
                    let mut parts = self.type_parts(&field_nodes(&member, "type"));
                    for c in children(&member) {
                        if c.kind() == "fixed_dimension" {
                            parts.postfix_dims.push(self.dim(&c));
                        }
                    }
                    es.fields.insert(
                        fname.clone(),
                        Field {
                            declaration: self.declaration(&fname, line_of(&member)),
                            r#type: parts.type_string(),
                        },
                    );
                }
                "enum_struct_method" => {
                    let mname = match self.name(&member) {
                        Some(n) => n,
                        None => continue,
                    };
                    es.methods.insert(
                        mname.clone(),
                        Function {
                            declaration: self.declaration(&mname, line_of(&member)),
                            kind: "stock".to_string(),
                            return_type: self.return_type(&member),
                            arguments: self.parameters(member.child_by_field_name("parameters")),
                        },
                    );
                }
                _ => {}
            }
        }
        self.strand.enumstructs.insert(name, es);
    }

    fn methodmap(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let parent = node
            .child_by_field_name("inherits")
            .map(|n| self.text(&n).to_string());

        let mut mm = MethodMap {
            declaration: self.declaration(&name, line_of(node)),
            parent,
            methods: HashMap::new(),
            properties: HashMap::new(),
        };

        for member in children(node) {
            let kind = member.kind();
            if kind == "methodmap_property" {
                if let Some(p) = self.property(&member) {
                    mm.properties.insert(p.declaration.name.clone(), p);
                }
                continue;
            }

            let mname = match self.name(&member) {
                Some(n) => n,
                None => continue,
            };
            let (method_kind, return_type, arguments) = match kind {
                "methodmap_native" => (
                    "native",
                    self.return_type(&member),
                    self.parameters(member.child_by_field_name("parameters")),
                ),
                "methodmap_method" => (
                    "stock",
                    self.return_type(&member),
                    self.parameters(member.child_by_field_name("parameters")),
                ),
                "methodmap_native_constructor" => (
                    "native",
                    name.clone(),
                    self.parameters(member.child_by_field_name("parameters")),
                ),
                "methodmap_method_constructor" => (
                    "stock",
                    name.clone(),
                    self.parameters(member.child_by_field_name("parameters")),
                ),
                "methodmap_native_destructor" => ("native", "void".to_string(), Vec::new()),
                "methodmap_method_destructor" => ("stock", "void".to_string(), Vec::new()),
                "methodmap_alias" => {
                    // `public Close() = CloseHandle;` borrows the aliased
                    // function's signature, minus the `this` argument
                    let target = member
                        .child_by_field_name("function")
                        .map(|f| self.text(&f).to_string());
                    match target.and_then(|t| self.strand.functions.get(&t)) {
                        Some(f) => (
                            "native",
                            f.return_type.clone(),
                            f.arguments.iter().skip(1).cloned().collect(),
                        ),
                        None => ("native", "void".to_string(), Vec::new()),
                    }
                }
                _ => continue,
            };
            let is_destructor = kind.ends_with("destructor");
            let mname = if is_destructor {
                format!("~{}", mname)
            } else {
                mname
            };
            mm.methods.insert(
                mname.clone(),
                Function {
                    declaration: self.declaration(&mname, line_of(&member)),
                    kind: method_kind.to_string(),
                    return_type,
                    arguments,
                },
            );
        }

        self.strand.methodmaps.insert(name, mm);
    }

    fn property(&self, node: &Node) -> Option<Property> {
        let name = self.name(node)?;
        let r#type = self.type_parts(&field_nodes(node, "type")).type_string();
        let mut getter = false;
        let mut setter = false;
        for accessor in children(node) {
            let mut stack = vec![accessor];
            while let Some(n) = stack.pop() {
                match n.kind() {
                    "methodmap_property_getter" | "get" => getter = true,
                    "methodmap_property_setter" | "set" => setter = true,
                    "block" => continue,
                    _ => stack.extend(children(&n)),
                }
            }
        }
        Some(Property {
            declaration: self.declaration(&name, line_of(node)),
            r#type,
            getter,
            setter,
        })
    }

    /// Renders a `typedef_expression` into `function <ret>(<args>)`
    fn typedef_expression(&self, node: &Node) -> String {
        let ret = self.return_type(node);
        let params = self.parameters(node.child_by_field_name("parameters"));
        self.signature(&ret, &params)
    }

    fn typedef(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let r#type = match children(node)
            .into_iter()
            .find(|c| c.kind() == "typedef_expression")
        {
            Some(e) => self.typedef_expression(&e),
            None => {
                let mut parts = self.type_parts(&field_nodes(node, "type"));
                // Dimensions after the aliased type are part of it
                parts.postfix_dims.clear();
                parts.type_string()
            }
        };
        self.insert_typedef(name, line_of(node), r#type);
    }

    fn insert_typedef(&mut self, name: String, line: usize, r#type: String) {
        let parsed_signature = parse_type_signature(&r#type);
        let td = TypeDefinition {
            declaration: self.declaration(&name, line),
            r#type,
            parsed_signature,
        };
        self.strand.typedefs.insert(name, td);
    }

    fn insert_typeset(&mut self, name: String, line: usize, types: Vec<(String, usize)>) {
        let mut ts = TypeSet {
            declaration: self.declaration(&name, line),
            types: HashMap::new(),
        };
        for (ty, line) in types {
            ts.types.insert(
                ty.clone(),
                Type {
                    documentation: self.documentation(line),
                    parsed_signature: parse_type_signature(&ty),
                    r#type: ty,
                },
            );
        }
        self.strand.typesets.insert(name, ts);
    }

    fn typeset(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let types = children(node)
            .iter()
            .filter(|c| c.kind() == "typedef_expression")
            .map(|c| (self.typedef_expression(c), line_of(c)))
            .collect();
        self.insert_typeset(name, line_of(node), types);
    }

    fn functag(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let ret = self.return_type(node);
        let params = self.parameters(node.child_by_field_name("parameters"));
        let sig = self.signature(&ret, &params);
        self.insert_typedef(name, line_of(node), sig);
    }

    fn funcenum(&mut self, node: &Node) {
        let name = match self.name(node) {
            Some(n) => n,
            None => return,
        };
        let types = children(node)
            .iter()
            .filter(|c| c.kind() == "funcenum_member")
            .map(|c| {
                let ret = self.return_type(c);
                let params = self.parameters(c.child_by_field_name("parameters"));
                (self.signature(&ret, &params), line_of(c))
            })
            .collect();
        self.insert_typeset(name, line_of(node), types);
    }

    fn defines(&mut self) {
        for m in self.pp.macros.iter().filter(|m| m.undefined_at.is_none()) {
            let line = self.line_of_offset(m.offset);
            let define = Define {
                declaration: self.declaration(&m.name, line),
                value: m.value.clone(),
            };
            self.strand.defines.insert(m.name.clone(), define);
        }
    }
}

/// Whether a function body contains `return <value>;`
fn returns_value(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut i = 0;
    while let Some(p) = body[i..].find("return") {
        let start = i + p;
        let end = start + "return".len();
        i = end;
        let bounded_left = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let bounded_right = end >= bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
        if !bounded_left || !bounded_right {
            continue;
        }
        match body[end..].trim_start().chars().next() {
            Some(';') | Some('}') | None => {}
            Some(_) => return true,
        }
    }
    false
}
