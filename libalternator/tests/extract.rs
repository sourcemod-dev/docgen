//! Focused tests of symbol extraction on small snippets

use alternator::parse;

#[test]
fn natives_forwards_and_notes() {
    let s = parse(
        r#"
/**
 * Closes a Handle.
 *
 * @note Closing a Handle has a different meaning for each Handle type.
 *
 * @param hndl          Handle to close.
 * @error               Invalid handles will cause a run time error.
 */
native void CloseHandle(Handle hndl);

/**
 * Called when the map starts.
 *
 * @note
 *   Notes on their own line are kept.
 */
forward void OnMapStart();
"#,
    )
    .unwrap();

    let close = &s.functions["CloseHandle"];
    assert_eq!(close.kind, "native");
    assert_eq!(close.return_type, "void");
    assert_eq!(close.declaration.documentation.ref_line, 10);
    let docs = close.declaration.documentation.docs.as_ref().unwrap();
    assert_eq!(docs.brief, "Closes a Handle.");
    assert_eq!(
        docs.tag("note"),
        Some("Closing a Handle has a different meaning for each Handle type.")
    );
    assert_eq!(docs.tag("param:hndl"), Some("Handle to close."));

    let start = &s.functions["OnMapStart"];
    assert_eq!(start.kind, "forward");
    let docs = start.declaration.documentation.docs.as_ref().unwrap();
    assert_eq!(docs.tag("note"), Some("Notes on their own line are kept."));
}

#[test]
fn arguments() {
    let s = parse(
        "native bool Foo(const char[] name, float vec[3], int &out, Handle &h, char buffer[MAX_LEN] = \"x\", int flags = (1<<2)|0x10, any ...);\n#define MAX_LEN 32\n",
    )
    .unwrap();

    let args: Vec<(&str, &str, Option<&str>)> = s.functions["Foo"]
        .arguments
        .iter()
        .map(|a| (a.r#type.as_str(), a.decl.as_str(), a.default.as_deref()))
        .collect();

    assert_eq!(
        args,
        vec![
            ("const char[]", "const char[] name", None),
            ("float[3]", "float vec[3]", None),
            ("int&", "int& out", None),
            ("Handle&", "Handle &h", None),
            ("char[MAX_LEN]", "char buffer[MAX_LEN]", Some("\"x\"")),
            ("int", "int flags", Some("1 << 2 | 16")),
            ("any...", "any...", None),
        ]
    );
}

#[test]
fn macros_expand_in_defaults() {
    let s = parse("#define FLAG_A (1<<0)\n#define FLAG_B 0x02\nnative void Foo(int flags = FLAG_A|FLAG_B);\n")
        .unwrap();
    assert_eq!(
        s.functions["Foo"].arguments[0].default.as_deref(),
        Some("1 << 0 | 2")
    );
    assert_eq!(s.defines["FLAG_A"].value, "(1<<0)");
}

#[test]
fn conditional_compilation() {
    let s = parse(
        r#"
#if defined _foo_included
 #endinput
#endif
#define _foo_included

#if !defined REQUIRE_PLUGIN
public void __pl_foo_SetNTVOptional()
{
    MarkNativeAsOptional("Foo");
}
#endif

#if defined SOMETHING_ELSE
native void Hidden();
#else
native void Shown();
#endif
"#,
    )
    .unwrap();

    assert!(s.functions.contains_key("__pl_foo_SetNTVOptional"));
    assert!(s.functions.contains_key("Shown"));
    assert!(!s.functions.contains_key("Hidden"));
    assert!(s.defines.contains_key("_foo_included"));
}

#[test]
fn methodmaps() {
    let s = parse(
        r#"
/** A list */
methodmap ArrayList < Handle {
    /** Creates a list */
    public native ArrayList(int blocksize=1);

    /** Clears it */
    public native void Clear();

    /** Length */
    property int Length {
        public native get();
    }

    property bool Flag {
        public get() { return true; }
        public set(bool value) { }
    }
};
"#,
    )
    .unwrap();

    let mm = &s.methodmaps["ArrayList"];
    assert_eq!(mm.parent.as_deref(), Some("Handle"));
    assert_eq!(mm.methods["ArrayList"].return_type, "ArrayList");
    assert_eq!(mm.methods["ArrayList"].arguments[0].default.as_deref(), Some("1"));
    assert_eq!(mm.methods["Clear"].kind, "native");
    assert_eq!(
        mm.methods["Clear"].declaration.documentation.docs.as_ref().unwrap().brief,
        "Clears it"
    );
    assert!(mm.properties["Length"].getter);
    assert!(!mm.properties["Length"].setter);
    assert!(mm.properties["Flag"].getter && mm.properties["Flag"].setter);
    assert_eq!(mm.properties["Flag"].r#type, "bool");
}

#[test]
fn enums_and_constants() {
    let s = parse(
        r#"
/** Actions */
enum Action
{
    Plugin_Continue = 0,   /**< Continue */
    Plugin_Changed = 1,    /**< Changed */
};

enum
{
    UNNAMED_A = (1<<1),
};

enum struct Point
{
    float x;
    char name[32];

    float Length() { return 0.0; }
}
"#,
    )
    .unwrap();

    let action = &s.enums["Action"];
    assert_eq!(action.entries["Plugin_Changed"].value.as_deref(), Some("1"));
    assert_eq!(
        action.entries["Plugin_Changed"]
            .declaration
            .documentation
            .docs
            .as_ref()
            .unwrap()
            .brief,
        "Changed"
    );
    assert!(s.constants.contains_key("UNNAMED_A"));

    let point = &s.enumstructs["Point"];
    assert_eq!(point.fields["name"].r#type, "char[32]");
    assert_eq!(point.methods["Length"].return_type, "float");
}

#[test]
fn typedefs_and_typesets() {
    let s = parse(
        r#"
typedef Timer = function Action (Handle timer, any data);
typedef Address = int64;

typeset SQLCallback
{
    /** Query */
    function void (Database db, DBResultSet results, const char[] error, any data);
    function void (Database db, any data);
};
"#,
    )
    .unwrap();

    let timer = &s.typedefs["Timer"];
    assert_eq!(timer.r#type, "function Action(Handle timer, any data)");
    let sig = timer.parsed_signature.as_ref().unwrap();
    assert_eq!(sig.return_type, "Action");
    assert_eq!(sig.arguments.len(), 2);

    assert_eq!(s.typedefs["Address"].r#type, "int64");
    assert!(s.typedefs["Address"].parsed_signature.is_none());

    let ts = &s.typesets["SQLCallback"];
    assert_eq!(ts.types.len(), 2);
    let query = &ts.types["function void(Database db, DBResultSet results, const char[] error, any data)"];
    assert_eq!(query.documentation.docs.as_ref().unwrap().brief, "Query");
}

#[test]
fn legacy_syntax() {
    let s = parse(
        r#"
native GetClientName(client, String:name[], maxlen);
native Float:GetGameTime();
functag public Action:Timer(Handle:timer);
public OnPluginStart()
{
    new String:yam[] = "yams";
}
"#,
    )
    .unwrap();

    let f = &s.functions["GetClientName"];
    assert_eq!(f.return_type, "int");
    assert_eq!(f.arguments[1].decl, "char[] name");
    assert_eq!(s.functions["GetGameTime"].return_type, "float");
    assert_eq!(s.functions["OnPluginStart"].return_type, "void");
    assert_eq!(s.typedefs["Timer"].r#type, "function Action(Handle timer)");
}

#[test]
fn statement_syntax_does_not_leak_into_declarations() {
    // Multiple statements per case aren't supported by the grammar; function
    // bodies are skipped so this must not produce bogus symbols
    let s = parse(
        r#"
enum struct Buffer {
    int cursor;

    void Write(int data) {
        switch (data) {
            case 0:
                this.cursor++;
                this.cursor++;
        }
    }

    int Read() { return 0; }
}
"#,
    )
    .unwrap();

    assert!(s.functions.is_empty());
    assert_eq!(s.enumstructs["Buffer"].methods.len(), 2);
}

#[test]
fn broken_declarations_fail() {
    assert!(parse("native void Foo(int a b c);\nnative void Bar();\n").is_err());
    assert!(parse("methodmap X {\n    public native void A(int a b);\n}\n").is_err());
}

#[test]
fn recoverable_errors_are_tolerated() {
    // Untyped legacy rest parameter, and a legacy global outside any declaration
    let s = parse("native Foo(a, ...);\nnew String:g_Name[] = \"x\";\n").unwrap();
    assert_eq!(s.functions["Foo"].arguments[1].r#type, "int...");
}
