// src/engine/mod.rs

use crate::domain::ComponentContext;
use handlebars::{
    Context, Handlebars, Helper, RenderContext, RenderError, RenderErrorReason, ScopedJson,
};
use serde_json::json;

#[derive(Default)]
pub struct TemplateEngine {
    registry: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut registry = Handlebars::new();

        // Register helpers
        registry.register_helper("interface_name", Box::new(interface_name_helper));
        registry.register_helper("mock_export_name", Box::new(mock_export_name_helper));

        // Strict mode: fail on missing variables
        registry.set_strict_mode(true);

        Self { registry }
    }

    pub fn render(
        &self,
        template: &str,
        context: &ComponentContext,
    ) -> Result<String, RenderError> {
        let data = json!(context);
        self.registry.render_template(template, &data)
    }
}

// Helper: PascalCase -> IBaseTemplate
fn interface_name_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn handlebars::Output,
) -> Result<(), RenderError> {
    let param = h
        .param(0)
        .ok_or(RenderErrorReason::InvalidParamType("param 0 required"))?;
    let name = param
        .value()
        .as_str()
        .ok_or(RenderErrorReason::MissingVariable(Some(
            "string required".to_string(),
        )))?;
    out.write(&format!("I{}", name))?;
    Ok(())
}

// Helper: PascalCase -> mockBaseTemplateProps
fn mock_export_name_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn handlebars::Output,
) -> Result<(), RenderError> {
    let param = h
        .param(0)
        .ok_or(RenderErrorReason::InvalidParamType("param 0 required"))?;
    let name = param
        .value()
        .as_str()
        .ok_or(RenderErrorReason::MissingVariable(Some(
            "string required".to_string(),
        )))?;
    let camel = to_camel_case(name);
    out.write(&format!("mock{}Props", capitalize_first(&camel)))?;
    Ok(())
}

fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ComponentContext;
    use handlebars::RenderError;

    // Helper to create a ComponentContext
    fn test_context(name: &str) -> ComponentContext {
        ComponentContext::new(name.to_string())
    }

    #[test]
    fn to_camel_case_works() {
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("U"), "u");
        assert_eq!(to_camel_case("UserCard"), "userCard");
        assert_eq!(to_camel_case("HTTPRequest"), "hTTPRequest");
        assert_eq!(to_camel_case("alreadyCamel"), "alreadyCamel");
    }

    #[test]
    fn capitalize_first_works() {
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("alreadyCapital"), "AlreadyCapital");
        assert_eq!(capitalize_first("HTTP"), "HTTP"); // first char already uppercase
    }

    #[test]
    fn interface_name_helper_works() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{interface_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "IButton");
    }

    #[test]
    fn interface_name_helper_with_custom_string() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Ignore"); // context not used directly because we pass literal
        let template = "{{interface_name \"User\"}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "IUser");
    }

    #[test]
    fn interface_name_helper_missing_param() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{interface_name}}";
        let err = engine.render(template, &ctx).unwrap_err();
        // The error message should indicate missing parameter
        assert!(err.to_string().contains("param 0 required"));
    }

    #[test]
    fn interface_name_helper_non_string_param() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        // Use a numeric literal (not a string)
        let template = "{{interface_name 123}}";
        let err = engine.render(template, &ctx).unwrap_err();
        assert!(err.to_string().contains("string required"));
    }

    #[test]
    fn mock_export_name_helper_works() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{mock_export_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "mockButtonProps");
    }

    #[test]
    fn mock_export_name_helper_with_pascal() {
        let engine = TemplateEngine::new();
        let template = "{{mock_export_name \"UserProfile\"}}";
        let ctx = test_context("irrelevant");
        let result = engine.render(template, &ctx).unwrap();
        // "UserProfile" -> camelCase "userProfile" -> capitalize "UserProfile" -> "mockUserProfileProps"
        assert_eq!(result, "mockUserProfileProps");
    }

    #[test]
    fn mock_export_name_helper_with_single_char() {
        let engine = TemplateEngine::new();
        let template = "{{mock_export_name \"X\"}}";
        let ctx = test_context("irrelevant");
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "mockXProps");
    }

    #[test]
    fn mock_export_name_helper_missing_param() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{mock_export_name}}";
        let err = engine.render(template, &ctx).unwrap_err();
        assert!(err.to_string().contains("param 0 required"));
    }

    #[test]
    fn mock_export_name_helper_non_string_param() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{mock_export_name true}}";
        let err = engine.render(template, &ctx).unwrap_err();
        assert!(err.to_string().contains("string required"));
    }

    #[test]
    fn render_simple_template() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "Hello {{component_name}} (camel: {{component_name_camel}})";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "Hello Button (camel: button)");
    }

    #[test]
    fn render_with_multiple_helpers() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "interface: {{interface_name component_name}}, mock: {{mock_export_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "interface: IButton, mock: mockButtonProps");
    }

    #[test]
    fn render_empty_template() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn helpers_handle_empty_string() {
        let engine = TemplateEngine::new();
        let ctx = test_context(""); // empty component name
        let template = "{{interface_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "I");
    }

    #[test]
    fn helpers_with_pure_whitespace_input() {
        let engine = TemplateEngine::new();
        let ctx = test_context("   ");
        let template = "{{mock_export_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        // Whitespace string -> to_camel_case preserves first char (space -> lowercased? but as_str gives "   ")
        // The helper uses param.value().as_str() which returns the string with spaces.
        // to_camel_case on "   " gives "   " (first char is space, which lowercasing does nothing).
        // Then capitalize_first on "   " gives "   " (first char space unchanged).
        // So final = "mock   Props". That's expected.
        assert_eq!(result, "mock   Props");
    }

    #[test]
    fn helpers_with_numbers_in_string() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button123");
        let template = "{{interface_name component_name}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "IButton123");
    }

    #[test]
    fn engine_creation_registers_helpers() {
        let engine = TemplateEngine::new();
        // If helpers weren't registered, these templates would fail with unknown helper error.
        let ctx = test_context("Test");
        let result_interface = engine.render("{{interface_name \"X\"}}", &ctx).unwrap();
        assert_eq!(result_interface, "IX");
        let result_mock = engine.render("{{mock_export_name \"X\"}}", &ctx).unwrap();
        assert_eq!(result_mock, "mockXProps");
    }

    #[test]
    fn strict_mode_errors_on_missing_variable() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Button");
        let template = "{{missing}}";
        let err = engine.render(template, &ctx).unwrap_err();

        use handlebars::RenderErrorReason;
        match err.reason() {
            RenderErrorReason::MissingVariable(name) => {
                assert_eq!(name.as_deref(), Some("missing"));
            }
            other => panic!("expected MissingVariable, got {:?}", other),
        }
    }

    #[test]
    fn engine_strict_mode_is_enabled() {
        let engine = TemplateEngine::new();
        let ctx = test_context("Test");
        let template = "{{nonexistent}}";
        let err = engine.render(template, &ctx).unwrap_err();

        use handlebars::RenderErrorReason;
        match err.reason() {
            RenderErrorReason::MissingVariable(name) => {
                assert_eq!(name.as_deref(), Some("nonexistent"));
            }
            other => panic!("expected MissingVariable, got {:?}", other),
        }
    }
}
