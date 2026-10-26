use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use std::path::Path;

pub fn typescript_to_javascript(source: &str) -> Result<String, String> {
    let allocator = Allocator::default();
    let path = Path::new("extension.ts");
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        let first = parsed
            .diagnostics
            .first()
            .map(|error| error.to_string())
            .unwrap_or_else(|| "parse error".to_owned());
        return Err(format!("typescript parse failed: {first}"));
    }
    let mut program = parsed.program;
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();
    let transformed = Transformer::new(&allocator, path, &TransformOptions::default())
        .build_with_scoping(scoping, &mut program);
    if let Some(error) = transformed.diagnostics.first() {
        return Err(format!("typescript transform failed: {error}"));
    }
    Ok(Codegen::new().build(&program).code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_types_interfaces_and_generics() {
        let out = typescript_to_javascript(
            "interface A { x: number }\nclass Provider<T> { name: string = 'a'; run(a: A): T | null { return null as any } }",
        )
        .unwrap();
        assert!(!out.contains("interface"));
        assert!(!out.contains(": number"));
        assert!(out.contains("class Provider"));
    }

    #[test]
    fn syntax_errors_are_reported() {
        assert!(typescript_to_javascript("class {").is_err());
    }
}
