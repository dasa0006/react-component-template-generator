use std::path::PathBuf;

/// The three files in every component package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateKind {
    Component,
    Mocks,
    Story,
}

impl TemplateKind {
    /// Returns filename template with {{component_name}} placeholder
    pub fn filename_pattern(&self) -> &'static str {
        match self {
            TemplateKind::Component => "{{component_name}}.tsx",
            TemplateKind::Mocks => "{{component_name}}.mocks.ts",
            TemplateKind::Story => "{{component_name}}.stories.tsx",
        }
    }

    /// Source template identifier
    pub fn source_key(&self) -> &'static str {
        match self {
            TemplateKind::Component => "component",
            TemplateKind::Mocks => "mocks",
            TemplateKind::Story => "story",
        }
    }

    /// All variants for iteration
    pub fn all() -> [TemplateKind; 3] {
        [
            TemplateKind::Component,
            TemplateKind::Mocks,
            TemplateKind::Story,
        ]
    }
}

/// Context passed to Handlebars templates
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentContext {
    pub component_name: String,
    pub component_name_camel: String,
}

impl ComponentContext {
    pub fn new(component_name: String) -> Self {
        Self {
            component_name_camel: to_camel_case(&component_name),
            component_name,
        }
    }
}

/// Final rendered output
pub struct GeneratedFile {
    pub kind: TemplateKind,
    pub content: String,
    pub filename: String, // e.g., "UserCard.tsx"
}

/// Location where files should be written
pub struct TargetLocation {
    pub base_path: PathBuf,         // /project/components/
    pub component_type_dir: String, // /ui
    pub component_dir: String,      // userCard
}

impl TargetLocation {
    pub fn full_path(&self) -> PathBuf {
        self.base_path.join(&self.component_dir)
    }
}

// Utility: PascalCase -> camelCase
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn template_kind_filename_pattern() {
        assert_eq!(TemplateKind::Component.filename_pattern(), "{{component_name}}.tsx");
        assert_eq!(TemplateKind::Mocks.filename_pattern(), "{{component_name}}.mocks.ts");
        assert_eq!(TemplateKind::Story.filename_pattern(), "{{component_name}}.stories.tsx");
    }

    #[test]
    fn template_kind_source_key() {
        assert_eq!(TemplateKind::Component.source_key(), "component");
        assert_eq!(TemplateKind::Mocks.source_key(), "mocks");
        assert_eq!(TemplateKind::Story.source_key(), "story");
    }

    #[test]
    fn template_kind_all() {
        let all = TemplateKind::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&TemplateKind::Component));
        assert!(all.contains(&TemplateKind::Mocks));
        assert!(all.contains(&TemplateKind::Story));
        // Optionally check order if required
        assert_eq!(all[0], TemplateKind::Component);
        assert_eq!(all[1], TemplateKind::Mocks);
        assert_eq!(all[2], TemplateKind::Story);
    }

    #[test]
    fn to_camel_case_works() {
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("U"), "u");
        assert_eq!(to_camel_case("UserCard"), "userCard");
        assert_eq!(to_camel_case("HTTPRequest"), "hTTPRequest"); // only first char lowercased
        assert_eq!(to_camel_case("userCard"), "userCard"); // already camelCase
        assert_eq!(to_camel_case("ABC"), "aBC");
    }

    #[test]
    fn component_context_new() {
        let ctx = ComponentContext::new("UserCard".to_string());
        assert_eq!(ctx.component_name, "UserCard");
        assert_eq!(ctx.component_name_camel, "userCard");

        let ctx_empty = ComponentContext::new("".to_string());
        assert_eq!(ctx_empty.component_name, "");
        assert_eq!(ctx_empty.component_name_camel, "");

        let ctx_single = ComponentContext::new("X".to_string());
        assert_eq!(ctx_single.component_name, "X");
        assert_eq!(ctx_single.component_name_camel, "x");
    }

    #[test]
    fn component_context_serialization() {
        // Verify that serde serialization works as expected (e.g., for Handlebars)
        let ctx = ComponentContext::new("Button".to_string());
        let json = serde_json::to_string(&ctx).expect("serialization failed");
        assert_eq!(json, r#"{"component_name":"Button","component_name_camel":"button"}"#);
    }

    #[test]
    fn target_location_full_path() {
        let loc = TargetLocation {
            base_path: PathBuf::from("/project/components"),
            component_type_dir: "ui".to_string(),
            component_dir: "button".to_string(),
        };
        // full_path should join base_path and component_dir (ignores component_type_dir as per implementation)
        assert_eq!(loc.full_path(), PathBuf::from("/project/components/button"));

        // Test with different path styles
        let loc_relative = TargetLocation {
            base_path: PathBuf::from("./src"),
            component_type_dir: "ui".to_string(),
            component_dir: "card".to_string(),
        };
        assert_eq!(loc_relative.full_path(), PathBuf::from("./src/card"));
    }

    #[test]
    fn generated_file_holds_data() {
        // Simple struct test – no logic, but ensures fields are accessible
        let file = GeneratedFile {
            kind: TemplateKind::Component,
            content: "some content".to_string(),
            filename: "Button.tsx".to_string(),
        };
        assert_eq!(file.kind, TemplateKind::Component);
        assert_eq!(file.content, "some content");
        assert_eq!(file.filename, "Button.tsx");
    }
}