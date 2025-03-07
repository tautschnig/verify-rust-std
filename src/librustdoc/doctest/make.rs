//! Logic for transforming the raw code given by the user into something actually
//! runnable, e.g. by adding a `main` function if it doesn't already exist.

use std::fmt::{self, Write as _};
use std::io;
use std::sync::Arc;

use rustc_ast::token::{Delimiter, TokenKind};
use rustc_ast::tokenstream::TokenTree;
use rustc_ast::{self as ast, HasAttrs, StmtKind};
use rustc_errors::ColorConfig;
use rustc_errors::emitter::stderr_destination;
use rustc_parse::new_parser_from_source_str;
use rustc_session::parse::ParseSess;
use rustc_span::edition::Edition;
use rustc_span::source_map::SourceMap;
use rustc_span::symbol::sym;
use rustc_span::{FileName, kw};
use tracing::debug;

use super::GlobalTestOptions;
use crate::display::Joined as _;
use crate::html::markdown::LangString;

#[derive(Default)]
struct ParseSourceInfo {
    has_main_fn: bool,
    found_extern_crate: bool,
    supports_color: bool,
    has_global_allocator: bool,
    has_macro_def: bool,
    everything_else: String,
    crates: String,
    crate_attrs: String,
    maybe_crate_attrs: String,
}

/// This struct contains information about the doctest itself which is then used to generate
/// doctest source code appropriately.
pub(crate) struct DocTestBuilder {
    pub(crate) supports_color: bool,
    pub(crate) already_has_extern_crate: bool,
    pub(crate) has_main_fn: bool,
    pub(crate) crate_attrs: String,
    /// If this is a merged doctest, it will be put into `everything_else`, otherwise it will
    /// put into `crate_attrs`.
    pub(crate) maybe_crate_attrs: String,
    pub(crate) crates: String,
    pub(crate) everything_else: String,
    pub(crate) test_id: Option<String>,
    pub(crate) failed_ast: bool,
    pub(crate) can_be_merged: bool,
}

impl DocTestBuilder {
    pub(crate) fn new(
        source: &str,
        crate_name: Option<&str>,
        edition: Edition,
        can_merge_doctests: bool,
        // If `test_id` is `None`, it means we're generating code for a code example "run" link.
        test_id: Option<String>,
        lang_str: Option<&LangString>,
    ) -> Self {
        let can_merge_doctests = can_merge_doctests
            && lang_str.is_some_and(|lang_str| {
                !lang_str.compile_fail && !lang_str.test_harness && !lang_str.standalone_crate
            });

        let result = rustc_driver::catch_fatal_errors(|| {
            rustc_span::create_session_if_not_set_then(edition, |_| {
                parse_source(source, &crate_name)
            })
        });

        let Ok(Ok(ParseSourceInfo {
            has_main_fn,
            found_extern_crate,
            supports_color,
            has_global_allocator,
            has_macro_def,
            everything_else,
            crates,
            crate_attrs,
            maybe_crate_attrs,
        })) = result
        else {
            return Self::invalid(
                String::new(),
                String::new(),
                String::new(),
                source.to_string(),
                test_id,
            );
        };

        debug!("crate_attrs:\n{crate_attrs}{maybe_crate_attrs}");
        debug!("crates:\n{crates}");
        debug!("after:\n{everything_else}");

        // If the AST returned an error, we don't want this doctest to be merged with the
        // others. Same if it contains `#[feature]` or `#[no_std]`.
        let can_be_merged = can_merge_doctests
            && !has_global_allocator
            && crate_attrs.is_empty()
            // If this is a merged doctest and a defined macro uses `$crate`, then the path will
            // not work, so better not put it into merged doctests.
            && !(has_macro_def && everything_else.contains("$crate"));
        Self {
            supports_color,
            has_main_fn,
            crate_attrs,
            maybe_crate_attrs,
            crates,
            everything_else,
            already_has_extern_crate: found_extern_crate,
            test_id,
            failed_ast: false,
            can_be_merged,
        }
    }

    fn invalid(
        crate_attrs: String,
        maybe_crate_attrs: String,
        crates: String,
        everything_else: String,
        test_id: Option<String>,
    ) -> Self {
        Self {
            supports_color: false,
            has_main_fn: false,
            crate_attrs,
            maybe_crate_attrs,
            crates,
            everything_else,
            already_has_extern_crate: false,
            test_id,
            failed_ast: true,
            can_be_merged: false,
        }
    }

    /// Transforms a test into code that can be compiled into a Rust binary, and returns the number of
    /// lines before the test code begins.
    pub(crate) fn generate_unique_doctest(
        &self,
        test_code: &str,
        dont_insert_main: bool,
        opts: &GlobalTestOptions,
        crate_name: Option<&str>,
    ) -> (String, usize) {
        if self.failed_ast {
            // If the AST failed to compile, no need to go generate a complete doctest, the error
            // will be better this way.
            debug!("failed AST:\n{test_code}");
            return (test_code.to_string(), 0);
        }
        let mut line_offset = 0;
        let mut prog = String::new();
        let everything_else = self.everything_else.trim();
        if opts.attrs.is_empty() {
            // If there aren't any attributes supplied by #![doc(test(attr(...)))], then allow some
            // lints that are commonly triggered in doctests. The crate-level test attributes are
            // commonly used to make tests fail in case they trigger warnings, so having this there in
            // that case may cause some tests to pass when they shouldn't have.
            prog.push_str("#![allow(unused)]\n");
            line_offset += 1;
        }

        // Next, any attributes that came from the crate root via #![doc(test(attr(...)))].
        for attr in &opts.attrs {
            prog.push_str(&format!("#![{attr}]\n"));
            line_offset += 1;
        }

        // Now push any outer attributes from the example, assuming they
        // are intended to be crate attributes.
        if !self.crate_attrs.is_empty() {
            prog.push_str(&self.crate_attrs);
            if !self.crate_attrs.ends_with('\n') {
                prog.push('\n');
            }
        }
        if !self.maybe_crate_attrs.is_empty() {
            prog.push_str(&self.maybe_crate_attrs);
            if !self.maybe_crate_attrs.ends_with('\n') {
                prog.push('\n');
            }
        }
        if !self.crates.is_empty() {
            prog.push_str(&self.crates);
            if !self.crates.ends_with('\n') {
                prog.push('\n');
            }
        }

        // Don't inject `extern crate std` because it's already injected by the
        // compiler.
        if !self.already_has_extern_crate &&
            !opts.no_crate_inject &&
            let Some(crate_name) = crate_name &&
            crate_name != "std" &&
            // Don't inject `extern crate` if the crate is never used.
            // NOTE: this is terribly inaccurate because it doesn't actually
            // parse the source, but only has false positives, not false
            // negatives.
            test_code.contains(crate_name)
        {
            // rustdoc implicitly inserts an `extern crate` item for the own crate
            // which may be unused, so we need to allow the lint.
            prog.push_str("#[allow(unused_extern_crates)]\n");

            prog.push_str(&format!("extern crate r#{crate_name};\n"));
            line_offset += 1;
        }

        // FIXME: This code cannot yet handle no_std test cases yet
        if dont_insert_main || self.has_main_fn || prog.contains("![no_std]") {
            prog.push_str(everything_else);
        } else {
            let returns_result = everything_else.ends_with("(())");
            // Give each doctest main function a unique name.
            // This is for example needed for the tooling around `-C instrument-coverage`.
            let inner_fn_name = if let Some(ref test_id) = self.test_id {
                format!("_doctest_main_{test_id}")
            } else {
                "_inner".into()
            };
            let inner_attr = if self.test_id.is_some() { "#[allow(non_snake_case)] " } else { "" };
            let (main_pre, main_post) = if returns_result {
                (
                    format!(
                        "fn main() {{ {inner_attr}fn {inner_fn_name}() -> core::result::Result<(), impl core::fmt::Debug> {{\n",
                    ),
                    format!("\n}} {inner_fn_name}().unwrap() }}"),
                )
            } else if self.test_id.is_some() {
                (
                    format!("fn main() {{ {inner_attr}fn {inner_fn_name}() {{\n",),
                    format!("\n}} {inner_fn_name}() }}"),
                )
            } else {
                ("fn main() {\n".into(), "\n}".into())
            };
            // Note on newlines: We insert a line/newline *before*, and *after*
            // the doctest and adjust the `line_offset` accordingly.
            // In the case of `-C instrument-coverage`, this means that the generated
            // inner `main` function spans from the doctest opening codeblock to the
            // closing one. For example
            // /// ``` <- start of the inner main
            // /// <- code under doctest
            // /// ``` <- end of the inner main
            line_offset += 1;

            prog.push_str(&main_pre);

            // add extra 4 spaces for each line to offset the code block
            if opts.insert_indent_space {
                write!(
                    prog,
                    "{}",
                    fmt::from_fn(|f| everything_else
                        .lines()
                        .map(|line| fmt::from_fn(move |f| write!(f, "    {line}")))
                        .joined("\n", f))
                )
                .unwrap();
            } else {
                prog.push_str(everything_else);
            };
            prog.push_str(&main_post);
        }

        debug!("final doctest:\n{prog}");

        (prog, line_offset)
    }
}

fn cancel_error_count(psess: &ParseSess) {
    // Reset errors so that they won't be reported as compiler bugs when dropping the
    // dcx. Any errors in the tests will be reported when the test file is compiled,
    // Note that we still need to cancel the errors above otherwise `Diag` will panic on
    // drop.
    psess.dcx().reset_err_count();
}

const DOCTEST_CODE_WRAPPER: &str = "fn f(){";

fn parse_source(source: &str, crate_name: &Option<&str>) -> Result<ParseSourceInfo, ()> {
    use rustc_errors::DiagCtxt;
    use rustc_errors::emitter::{Emitter, HumanEmitter};
    use rustc_span::source_map::FilePathMapping;

    let mut info =
        ParseSourceInfo { found_extern_crate: crate_name.is_none(), ..Default::default() };

    let wrapped_source = format!("{DOCTEST_CODE_WRAPPER}{source}\n}}");

    let filename = FileName::anon_source_code(&wrapped_source);

    // Any errors in parsing should also appear when the doctest is compiled for real, so just
    // send all the errors that librustc_ast emits directly into a `Sink` instead of stderr.
    let sm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let fallback_bundle = rustc_errors::fallback_fluent_bundle(
        rustc_driver::DEFAULT_LOCALE_RESOURCES.to_vec(),
        false,
    );
    info.supports_color =
        HumanEmitter::new(stderr_destination(ColorConfig::Auto), fallback_bundle.clone())
            .supports_color();

    let emitter = HumanEmitter::new(Box::new(io::sink()), fallback_bundle);

    // FIXME(misdreavus): pass `-Z treat-err-as-bug` to the doctest parser
    let dcx = DiagCtxt::new(Box::new(emitter)).disable_warnings();
    let psess = ParseSess::with_dcx(dcx, sm);

    let mut parser = match new_parser_from_source_str(&psess, filename, wrapped_source) {
        Ok(p) => p,
        Err(errs) => {
            errs.into_iter().for_each(|err| err.cancel());
            cancel_error_count(&psess);
            return Err(());
        }
    };

    fn push_to_s(
        s: &mut String,
        source: &str,
        span: rustc_span::Span,
        prev_span_hi: &mut Option<usize>,
    ) {
        let extra_len = DOCTEST_CODE_WRAPPER.len();
        // We need to shift by 1 because we added `{` at the beginning of the source.we provided
        // to the parser.
        let lo = prev_span_hi.unwrap_or(0);
        let mut hi = span.hi().0 as usize - extra_len;
        if hi > source.len() {
            hi = source.len();
        }
        s.push_str(&source[lo..hi]);
        *prev_span_hi = Some(hi);
    }

    // Recurse through functions body. It is necessary because the doctest source code is
    // wrapped in a function to limit the number of AST errors. If we don't recurse into
    // functions, we would thing all top-level items (so basically nothing).
    fn check_item(
        item: &ast::Item,
        info: &mut ParseSourceInfo,
        crate_name: &Option<&str>,
        is_top_level: bool,
    ) -> bool {
        let mut is_crate = false;
        if !info.has_global_allocator
            && item.attrs.iter().any(|attr| attr.name_or_empty() == sym::global_allocator)
        {
            info.has_global_allocator = true;
        }
        match item.kind {
            ast::ItemKind::Fn(ref fn_item) if !info.has_main_fn => {
                // We only push if it's the top item because otherwise, we would duplicate
                // its content since the top-level item was already added.
                if item.ident.name == sym::main && is_top_level {
                    info.has_main_fn = true;
                }
                if let Some(ref body) = fn_item.body {
                    for stmt in &body.stmts {
                        if let StmtKind::Item(ref item) = stmt.kind {
                            is_crate |= check_item(item, info, crate_name, false);
                        }
                    }
                }
            }
            ast::ItemKind::ExternCrate(original) => {
                is_crate = true;
                if !info.found_extern_crate
                    && let Some(crate_name) = crate_name
                {
                    info.found_extern_crate = match original {
                        Some(name) => name.as_str() == *crate_name,
                        None => item.ident.as_str() == *crate_name,
                    };
                }
            }
            ast::ItemKind::MacroDef(..) => {
                info.has_macro_def = true;
            }
            _ => {}
        }
        is_crate
    }

    let mut prev_span_hi = None;
    let not_crate_attrs = [sym::forbid, sym::allow, sym::warn, sym::deny];
    let parsed = parser.parse_item(rustc_parse::parser::ForceCollect::No);

    let result = match parsed {
        Ok(Some(ref item))
            if let ast::ItemKind::Fn(ref fn_item) = item.kind
                && let Some(ref body) = fn_item.body =>
        {
            for attr in &item.attrs {
                let attr_name = attr.name_or_empty();

                if attr.style == ast::AttrStyle::Outer || not_crate_attrs.contains(&attr_name) {
                    // There is one exception to these attributes:
                    // `#![allow(internal_features)]`. If this attribute is used, we need to
                    // consider it only as a crate-level attribute.
                    if attr_name == sym::allow
                        && let Some(list) = attr.meta_item_list()
                        && list.iter().any(|sub_attr| {
                            sub_attr.name_or_empty().as_str() == "internal_features"
                        })
                    {
                        push_to_s(&mut info.crate_attrs, source, attr.span, &mut prev_span_hi);
                    } else {
                        push_to_s(
                            &mut info.maybe_crate_attrs,
                            source,
                            attr.span,
                            &mut prev_span_hi,
                        );
                    }
                } else {
                    push_to_s(&mut info.crate_attrs, source, attr.span, &mut prev_span_hi);
                }
            }
            for stmt in &body.stmts {
                let mut is_crate = false;
                match stmt.kind {
                    StmtKind::Item(ref item) => {
                        is_crate = check_item(&item, &mut info, crate_name, true);
                    }
                    StmtKind::Expr(ref expr) if matches!(expr.kind, ast::ExprKind::Err(_)) => {
                        cancel_error_count(&psess);
                        return Err(());
                    }
                    StmtKind::MacCall(ref mac_call) if !info.has_main_fn => {
                        let mut iter = mac_call.mac.args.tokens.iter();

                        while let Some(token) = iter.next() {
                            if let TokenTree::Token(token, _) = token
                                && let TokenKind::Ident(name, _) = token.kind
                                && name == kw::Fn
                                && let Some(TokenTree::Token(fn_token, _)) = iter.peek()
                                && let TokenKind::Ident(fn_name, _) = fn_token.kind
                                && fn_name == sym::main
                                && let Some(TokenTree::Delimited(_, _, Delimiter::Parenthesis, _)) = {
                                    iter.next();
                                    iter.peek()
                                }
                            {
                                info.has_main_fn = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }

                // Weirdly enough, the `Stmt` span doesn't include its attributes, so we need to
                // tweak the span to include the attributes as well.
                let mut span = stmt.span;
                if let Some(attr) = stmt.kind.attrs().first() {
                    span = span.with_lo(attr.span.lo());
                }
                if info.everything_else.is_empty()
                    && (!info.maybe_crate_attrs.is_empty() || !info.crate_attrs.is_empty())
                {
                    // We add potential backlines/comments if there are some in items generated
                    // before the wrapping function.
                    push_to_s(&mut info.crates, source, span.shrink_to_lo(), &mut prev_span_hi);
                }
                if !is_crate {
                    push_to_s(&mut info.everything_else, source, span, &mut prev_span_hi);
                } else {
                    push_to_s(&mut info.crates, source, span, &mut prev_span_hi);
                }
            }
            Ok(info)
        }
        Err(e) => {
            e.cancel();
            Err(())
        }
        _ => Err(()),
    };

    cancel_error_count(&psess);
    result
}
