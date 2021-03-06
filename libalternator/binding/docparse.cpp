// vim: set ts=2 sw=2 tw=99 et:
// 
// Copyright (C) 2012-2014 David Anderson
// 
// This file is part of SourcePawn.
// 
// SourcePawn is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
// 
// SourcePawn is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License along with
// SourcePawn. If not, see http://www.gnu.org/licenses/.
#include <iostream>
#include <sstream>
#include "shared/string-pool.h"
#include "compiler/reporting.h"
#include "compiler/source-manager.h"
#include "compiler/compile-context.h"
#include "compiler/parser/preprocessor.h"
#include "compiler/parser/parser.h"
#include "compiler/parser/json-tools.h"
#include "compiler/sema/name-resolver.h"
#include <assert.h>
#include <amtl/experimental/am-argparser.h>
#include "docparse.h"

using namespace ke;
using namespace sp;

// Comment analyzer. This is designed to only work when we're ignoring
// #include directives.
class Comments : public CommentDelegate
{
  struct CommentLoc {
    unsigned line;
    unsigned offset;

    CommentLoc()
     : line(0),
       offset(0)
    {}
    CommentLoc(unsigned line, unsigned offset)
     : line(line),
       offset(offset)
    {}
  };
  struct CommentRange {
    CommentLoc start;
    CommentLoc end;

    CommentRange()
    {}
    CommentRange(const CommentLoc &start, const CommentLoc &end)
     : start(start),
       end(end)
    {}
  };

 public:
  Comments(CompileContext &cc)
   : cc_(cc)
  {}

  void HandleComment(CommentPos aWhere, const SourceRange &aRange)
  {
    Vector<CommentRange> &where = (aWhere == CommentPos::Front)
                                  ? lead_comments_
                                  : tail_comments_;

    FullSourceRef start = cc_.source().decode(aRange.start);
    FullSourceRef end = cc_.source().decode(aRange.end);
    assert(start.file && end.file && start.file == end.file);

    CommentRange range(
      CommentLoc(start.line, start.offset),
      CommentLoc(end.line, end.offset));

    if (!where.empty()) {
      // Comments must be in order.
      assert(range.start.offset >= where.back().end.offset);
    }
    where.append(range);
  }

  static int cmp_ends_at_line(const void *aItem1, const void *aItem2) {
    unsigned line = reinterpret_cast<uintptr_t>(aItem1);
    const CommentRange *item2 = (const CommentRange *)aItem2;
    if (line < item2->end.line)
      return -1;
    if (line > item2->end.line)
      return 1;
    return 0;
  }

  static int cmp_starts_at_line(const void *aItem1, const void *aItem2) {
    unsigned line = reinterpret_cast<uintptr_t>(aItem1);
    const CommentRange *item2 = (const CommentRange *)aItem2;
    if (line < item2->start.line)
      return -1;
    if (line > item2->start.line)
      return 1;
    return 0;
  }

  bool findCommentFor(const SourceLocation &loc, unsigned *start, unsigned *end, unsigned *line) {
    // Find a comment that ends one line above loc.
    FullSourceRef ref = cc_.source().decode(loc);
    assert(ref.file);

    *line = ref.line;

    void *found =
      bsearch(reinterpret_cast<void *>(ref.line - 1),
              lead_comments_.buffer(),
              lead_comments_.length(),
              sizeof(CommentRange),
              cmp_ends_at_line);
    if (!found) {
      found = bsearch(reinterpret_cast<void *>(ref.line),
                      tail_comments_.buffer(),
                      tail_comments_.length(),
                      sizeof(CommentRange),
                      cmp_starts_at_line);
      if (!found)
        return false;
    }

    CommentRange *range = reinterpret_cast<CommentRange *>(found);
    *start = range->start.offset;
    *end = range->end.offset;
    return true;
  }

 private:
  CompileContext &cc_;
  Vector<CommentRange> lead_comments_;
  Vector<CommentRange> tail_comments_;
};

class ExprToStr : public StrictAstVisitor
{
 public:
  static ke::AString Convert(Expression *expr) {
    ExprToStr converter(expr);
    return converter.result_;
  }

 private:
  ExprToStr(Expression *expr) {
    expr->accept(this);
  }

  void visitIntegerLiteral(IntegerLiteral *node) override {
    result_.format("%" KE_FMT_I64, node->value());
  }
  void visitFloatLiteral(FloatLiteral *node) override {
    result_.format("%f", node->value());
  }
  void visitStringLiteral(StringLiteral *node) override {
    result_.format("\"%s\"", node->literal()->chars());
  }
  void visitCharLiteral(CharLiteral *node) override {
    result_.format("'%c'", (char)node->value());
  }
  void visitBinaryExpression(BinaryExpression *node) override {
    ke::AString left = Convert(node->left());
    ke::AString right = Convert(node->right());
    result_.format("%s %s %s", left.chars(), TokenNames[node->token()], right.chars());
  }
  void visitAssignment(Assignment *node) override {
    ke::AString lvalue = Convert(node->lvalue());
    ke::AString rvalue = Convert(node->expression());
    result_.format("%s %s %s", lvalue.chars(), TokenNames[node->token()], rvalue.chars());
  }
  void visitNameProxy(NameProxy *node) override {
    result_ = ke::AString(node->name()->chars(), node->name()->length());
  }
  void visitCallExpression(CallExpression *node) override {
    ke::AString callee = Convert(node->callee());
    result_.format("%s(", callee.chars());
    ExpressionList *args = node->arguments();
    for (size_t i = 0; i < args->length(); i++) {
      ke::AString arg = Convert(args->at(i));
      if (i != args->length() - 1)
        result_.format("%s%s, ", result_.chars(), arg.chars());
      else
        result_.format("%s%s", result_.chars(), arg.chars());
    }
    result_.format("%s)", result_.chars());
  }
  void visitFieldExpression(FieldExpression *node) override {
    ke::AString base = Convert(node->base());
    result_.format("%s.%s", base.chars(), node->field()->chars());
  }
  void visitIndexExpression(IndexExpression *node) override {
    ke::AString left = Convert(node->left());
    ke::AString right = Convert(node->right());
    result_.format("%s[%s]", left.chars(), right.chars());
  }
  void visitIncDecExpression(IncDecExpression *node) override {
    ke::AString expr = Convert(node->expression());
    if (node->postfix())
      result_.format("%s%s", expr.chars(), TokenNames[node->token()]);
    else
      result_.format("%s%s", TokenNames[node->token()], expr.chars());
  }
  void visitUnaryExpression(UnaryExpression *node) override {
    ke::AString expr = Convert(node->expression());
    result_.format("%s%s", TokenNames[node->token()], expr.chars());
  }
  void visitViewAsExpression(ViewAsExpression *node) override {
    ke::AString type = BuildTypeName(node->te(), nullptr);
    ke::AString expr = Convert(node->expr());
    result_.format("view_as<%s>(%s)", type.chars(), expr.chars());
  }
  void visitSizeofExpression(SizeofExpression *node) override {
    Atom *atom_name = node->proxy()->name();
    result_.format("sizeof(%s", atom_name->chars());
    AbstractAccessorExpression* accessor = node->accessorExpression();
    while (accessor != nullptr) {
      if (AbstractArrayMemberExpression* arrayExpr = accessor->asAbstractArrayMemberExpression()) {
        for (size_t i = 0; i < arrayExpr->level(); i++) {
          result_.format("%s[]", result_.chars());
        }
      } else if (AbstractFieldExpression* fieldExpr = accessor->asAbstractFieldExpression()) {
        Atom *field_name = fieldExpr->field();
        TokenKind token = fieldExpr->token();
        result_.format("%s%s%s", result_.chars(), TokenNames[token], field_name->chars());
      }
      accessor = accessor->child();
    }
    
    result_.format("%s)", result_.chars());
  }
  void visitTernaryExpression(TernaryExpression *node) override {
    ke::AString condition = Convert(node->condition());
    ke::AString left = Convert(node->left());
    ke::AString right = Convert(node->right());
    result_.format("%s ? %s : %s", condition.chars(), left.chars(), right.chars());
  }
  void visitTokenLiteral(TokenLiteral *node) override {
    result_ = TokenNames[node->token()];
  }
  void visitArrayLiteral(ArrayLiteral *node) override {
    if (node->arrayLength() == 0) {
      result_ = "{}";
      return;
    }

    ExpressionList* exprs = node->expressions();

    result_ = "{ ";
    for (size_t i = 0; i < exprs->length(); i++) {
      ke::AString element = Convert(exprs->at(i));
      if (i == exprs->length() - 1 && !node->repeatLastElement())
        result_.format("%s%s", result_.chars(), element.chars());
      else
        result_.format("%s%s, ", result_.chars(), element.chars());
    }
    if (node->repeatLastElement())
      result_.format("%s...", result_.chars());
    result_.format("%s }", result_.chars());
  }
  void visitThisExpression(ThisExpression* node) override {
    result_ = "this";
  }

 private:
  ke::AString result_;
};

class Analyzer : public PartialAstVisitor
{
 public:
  Analyzer(CompileContext &cc, Comments &comments)
   : cc_(cc),
     pool_(cc.pool()),
     comments_(comments)
  {
    atom_name_ = cc_.add("name");
    atom_kind_ = cc_.add("kind");
    atom_returnType_ = cc_.add("returnType");
    atom_type_ = cc_.add("type");
    atom_parameters_ = cc_.add("arguments");
    atom_ref_line_ = cc_.add("refLine");
    atom_doc_start_ = cc_.add("docStart");
    atom_doc_end_ = cc_.add("docEnd");
    atom_properties_ = cc_.add("properties");
    atom_methods_ = cc_.add("methods");
    atom_fields_ = cc_.add("fields");
    atom_getter_ = cc_.add("getter");
    atom_setter_ = cc_.add("setter");
    atom_entries_ = cc_.add("entries");
    atom_constants_ = cc_.add("constants");
    atom_decl_ = cc_.add("decl");
    atom_default_ = cc_.add("default");
    atom_value_ = cc_.add("value");
    atom_parent_ = cc_.add("parent");
  }

  JsonObject *analyze(ParseTree *tree, Preprocessor& pp) {
    functions_ = new (pool_) JsonList();
    methodmaps_ = new (pool_) JsonList();
    enums_ = new (pool_) JsonList();
    constants_ = new (pool_) JsonList();
    typesets_ = new (pool_) JsonList();
    typedefs_ = new (pool_) JsonList();
    enum_structs_ = new (pool_) JsonList();
    defines_ = new (pool_) JsonList();

    for (auto iter = pp.macros(); !iter.empty(); iter.next()) {
      visitMacro(iter->key, iter->value);
    }
    
    for (size_t i = 0; i < tree->statements()->length(); i++) {
      Statement *stmt = tree->statements()->at(i);
      stmt->accept(this);
    }

    JsonObject *obj = new (pool_) JsonObject();
    obj->add(cc_.add("functions"), functions_);
    obj->add(cc_.add("methodmaps"), methodmaps_);
    obj->add(cc_.add("enums"), enums_);
    obj->add(cc_.add("constants"), constants_);
    obj->add(cc_.add("typesets"), typesets_);
    obj->add(cc_.add("typedefs"), typedefs_);
    obj->add(cc_.add("enumstructs"), enum_structs_);
    obj->add(cc_.add("defines"), defines_);
    return obj;
  }

  void visitRecordDecl(RecordDecl* node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "enum struct", node->name(), node->loc());

    SaveAndSet<JsonList *> new_props(&props_, new (pool_) JsonList());
    SaveAndSet<JsonList *> new_methods(&methods_, new (pool_) JsonList());
    for (size_t i = 0; i < node->body()->length(); i++)
      node->body()->at(i)->accept(this);

    obj->add(atom_methods_, methods_);
    obj->add(atom_fields_, props_);
    enum_structs_->add(obj);
  }

  void visitMethodmapDecl(MethodmapDecl *node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "class", node->name(), node->loc());

    SaveAndSet<JsonList *> new_props(&props_, new (pool_) JsonList());
    SaveAndSet<JsonList *> new_methods(&methods_, new (pool_) JsonList());
    for (size_t i = 0; i < node->body()->length(); i++)
      node->body()->at(i)->accept(this);

    obj->add(atom_methods_, methods_);
    obj->add(atom_properties_, props_);
    if (NameProxy* parent = node->parent()) {
      Atom* parent_name = parent->name();
      obj->add(atom_parent_, toJson(parent_name->chars()));
    }
    methodmaps_->add(obj);
  }

  void visitMethodDecl(MethodDecl *node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "method", node->name(), node->loc());

    FunctionNode *fun = node->method();
    if (fun->signature()->native())
        obj->add(atom_kind_, toJson("native"));
    else
        obj->add(atom_kind_, toJson("stock"));
    obj->add(atom_returnType_, toJson(fun->signature()->returnType()));
    obj->add(atom_parameters_, toJson(fun->signature()->parameters()));
    methods_->add(obj);
  }
  void visitPropertyDecl(PropertyDecl *node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "property", node->name(), node->loc());

    obj->add(atom_type_, toJson(node->te()));
    obj->add(atom_getter_, new (pool_) JsonBool(!!node->getter()));
    obj->add(atom_setter_, new (pool_) JsonBool(!!node->setter()));
    props_->add(obj);
  }
  void visitFieldDecl(FieldDecl* node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "field", node->name(), node->loc());

    obj->add(atom_type_, toJson(node->te()));
    props_->add(obj);
  }

  void visitTypesetDecl(TypesetDecl *decl) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(decl->name()));
    startDoc(obj, "typeset", decl->name(), decl->loc());

    JsonList *list = new (pool_) JsonList();
    for (size_t i = 0; i < decl->types()->length(); i++) {
      const TypesetDecl::Entry &entry = decl->types()->at(i);
      JsonObject *te = new (pool_) JsonObject();
      te->add(atom_type_, toJson(entry.te));
      unsigned start, end, line;
      if (comments_.findCommentFor(entry.loc, &start, &end, &line)) {
        te->add(atom_ref_line_, new (pool_) JsonInt(line));
        te->add(atom_doc_start_, new (pool_) JsonInt(start));
        te->add(atom_doc_end_, new (pool_) JsonInt(end));
      }
      list->add(te);
    }
    obj->add(cc_.add("types"), list);

    typesets_->add(obj);
  }

  void visitTypedefDecl(TypedefDecl *decl) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(decl->name()));
    startDoc(obj, "typedef", decl->name(), decl->loc());

    obj->add(atom_type_, toJson(decl->te()));

    typedefs_->add(obj);
  }

  void visitEnumStatement(EnumStatement *node) override {
    if (!node->name()) {
      for (size_t i = 0; i < node->entries()->length(); i++) {
        EnumConstant *cs = node->entries()->at(i);

        JsonObject *val = new (pool_) JsonObject();
        val->add(atom_name_, toJson(cs->name()));
        if (cs->expression()) {
          ke::AString expr = ExprToStr::Convert(cs->expression());
          val->add(atom_value_, toJson(expr.chars()));
        }
        startDoc(val, "enum value", cs->name(), cs->loc());

        constants_->add(val);
      }
      return;
    }

    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "enum", node->name(), node->loc());

    JsonList *list = new (pool_) JsonList();
    for (size_t i = 0; i < node->entries()->length(); i++) {
      EnumConstant *cs = node->entries()->at(i);

      JsonObject *val = new (pool_) JsonObject();
      val->add(atom_name_, toJson(cs->name()));
      if (cs->expression()) {
        ke::AString expr = ExprToStr::Convert(cs->expression());
        val->add(atom_value_, toJson(expr.chars()));
      }
      startDoc(val, "enum value", cs->name(), cs->loc());

      list->add(val);
    }
    obj->add(atom_entries_, list);

    enums_->add(obj);
  }

  void visitFunctionStatement(FunctionStatement *node) override {
    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(node->name()));
    startDoc(obj, "function", node->name(), node->loc());

    obj->add(atom_kind_, toJson(node->decoration()));
    obj->add(atom_returnType_, toJson(node->signature()->returnType()));
    obj->add(atom_parameters_, toJson(node->signature()->parameters()));

    functions_->add(obj);
  }

  void visitMacro(Atom* key, Macro *node) {
    // Ignore macros that aren't backed by file (e.g. __sourcepawn__)
    FullSourceRef ref = cc_.source().decode(node->start());
    if (!ref.file)
      return;

    JsonObject *obj = new (pool_) JsonObject();
    obj->add(atom_name_, toJson(key));
    startDoc(obj, "define", key, node->start());

    std::string value;
    if (node->tokens->length()) {
      ref = cc_.source().decode(node->start());
      if (ref.file) {
        const char* file = ref.file->chars();

        value = std::string{
          file + ref.offset,
          file + ref.offset + node->length(),
        };
      }
    }

    obj->add(atom_value_, toJson(value.c_str()));

    defines_->add(obj);
  }

 private:
  void startDoc(JsonObject *obj, const char *type, Atom *name, const SourceLocation &loc) {
    unsigned start, end, line;
    if (!comments_.findCommentFor(loc, &start, &end, &line)) {
      cc_.report(loc, rmsg::missing_comment)
        << type << name;
      return;
    }

    assert(start < INT_MAX);
    assert(end < INT_MAX);

    obj->add(atom_ref_line_, new (pool_) JsonInt(line));
    obj->add(atom_doc_start_, new (pool_) JsonInt(start));
    obj->add(atom_doc_end_, new (pool_) JsonInt(end));
  }

  JsonString *toJson(const TypeSpecifier *spec, Atom *name = nullptr) {
    return toJson(BuildTypeName(spec, name, TypeDiagFlags::Names).chars());
  }
  JsonString *toJson(Type *type, Atom *name = nullptr) {
    return toJson(BuildTypeName(type, name, TypeDiagFlags::Names).chars());
  }

  JsonString *toJson(const TypeExpr &te, Atom *name = nullptr) {
    if (te.spec())
      return toJson(te.spec(), name);
    return toJson(te.resolved(), name);
  }

  static inline bool isByRef(const TypeExpr& te) {
    return te.resolved()
           ? te.resolved()->isReference()
           : te.spec()->isByRef();
  }
  static inline bool isConst(const TypeExpr& te) {
    return te.resolved()
           ? te.resolved()->isConst()
           : te.spec()->isConst();
  }

  JsonString *toJson(VarDecl *decl, bool named) {
    // :TODO: add a BuildTypeName(VarDecl) helper.
    TypeDiagFlags flags = TypeDiagFlags::Names;
    if (isByRef(decl->te()))
      flags |= TypeDiagFlags::IsByRef;
    if (isConst(decl->te()))
      flags |= TypeDiagFlags::IsConst;
    return toJson(BuildTypeName(
      decl->te(),
      named ? decl->name() : nullptr,
      flags).chars());
  }

  JsonList *toJson(const ParameterList *params) {
    JsonList *list = new (pool_) JsonList();
    for (size_t i = 0; i < params->length(); i++) {
      VarDecl *decl = params->at(i);
      JsonObject *obj = new (pool_) JsonObject();

      obj->add(atom_type_, toJson(decl, false));

      if (decl->name()) {
        obj->add(atom_name_, toJson(decl->name()));
        obj->add(atom_decl_, toJson(decl, true));
        if (decl->initialization()) {
          ke::AString def = ExprToStr::Convert(decl->initialization());
          obj->add(atom_default_, toJson(def.chars()));
        }
      } else {
        obj->add(atom_name_, toJson("..."));

        AutoString builder = BuildTypeName(decl->te(), nullptr, TypeDiagFlags::Names);
        obj->add(atom_decl_, toJson(builder.ptr()));
      }
      list->add(obj);
    }
    return list;
  }

  JsonString *toJson(Atom *name) {
    return new (pool_) JsonString(name);
  }
  JsonString *toJson(const char *str) {
    return new (pool_) JsonString(cc_.add(str));
  }

 private:
  CompileContext &cc_;
  PoolAllocator &pool_;
  Comments &comments_;

  Atom *atom_name_;
  Atom *atom_kind_;
  Atom *atom_returnType_;
  Atom *atom_type_;
  Atom *atom_parameters_;
  Atom *atom_ref_line_;
  Atom *atom_doc_start_;
  Atom *atom_doc_end_;
  Atom *atom_properties_;
  Atom *atom_methods_;
  Atom *atom_fields_;
  Atom *atom_getter_;
  Atom *atom_setter_;
  Atom *atom_entries_;
  Atom *atom_constants_;
  Atom *atom_decl_;
  Atom *atom_default_;
  Atom *atom_value_;
  Atom *atom_parent_;
  JsonList *functions_;
  JsonList *methodmaps_;
  JsonList *enums_;
  JsonList *constants_;
  JsonList *typesets_;
  JsonList *typedefs_;
  JsonList *enum_structs_;
  JsonList *defines_;

  JsonList *props_;
  JsonList *methods_;
};

static JsonObject *
Run(CompileContext &cc, const char* buffer, const char* path)
{
  Comments comments(cc);
  ParseTree *tree = nullptr;
  
  Preprocessor pp(cc);

  pp.disableIncludes();
  pp.setCommentDelegate(&comments);

  UniquePtr<char[]> o_ptr = MakeUnique<char[]>(strlen(buffer) + 10);
  strcpy(o_ptr.get(), buffer);

  {
    RefPtr<SourceFile> file = cc.source().createFromBuffer(std::move(o_ptr), strlen(buffer), path);

    if (!file)
      return nullptr;
    if (!pp.enter(file))
      return nullptr;
  }
  
  NameResolver nr(cc);
  Parser parser(cc, pp, nr);

  tree = parser.parse();
  if (!tree || !cc.phasePassed())
    return nullptr;

  Analyzer analyzer(cc, comments);
  return analyzer.analyze(tree, pp);
}

const char* parse(const char* input, const char* path) {
    StringPool strings;
    ReportManager reports;
    SourceManager source(strings, reports);

    PoolAllocator pool;
    {
        PoolScope scope(pool);

        CompileContext cc(pool, strings, reports, source);

        cc.SkipResolution();

        JsonObject *obj = Run(cc, input, path);

        if (!obj) {
          return nullptr;
        }

        std::stringstream out;

        JsonRenderer renderer(out);
        renderer.Render(obj);

        char *optr = new char[out.str().size()+1];
        strcpy(optr, out.str().c_str());

        return optr;
    }
}
