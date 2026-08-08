use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    CompiledProject, GeneratedProjectModule, ProjectModuleIdentity, RojoMappingField,
    RojoMappingProblem, RojoMappingRejection,
};

/// Writes a deterministic Rojo project mapping for one accepted generated project.
///
/// The generated-root value is a relative slash-separated path from the Rojo project file to
/// the tree written by `write_compiled_project_atomically`. Rojo is never invoked by this API.
///
/// # Errors
///
/// Returns a typed rejection when configuration is invalid, two modules claim one instance path,
/// or the destination cannot be written.
pub fn write_rojo_project_mapping(
    mapping_parts: (&CompiledProject, &str, &str, impl AsRef<Path>),
) -> Result<(), RojoMappingRejection> {
    let (compiled_project, project_name, generated_root, destination_path) = mapping_parts;
    let destination_path = destination_path.as_ref();
    if project_name.trim().is_empty() {
        return Err(rejection((
            RojoMappingField::ProjectName,
            None,
            RojoMappingProblem::EmptyValue,
            destination_path,
        )));
    }
    if !valid_generated_root(generated_root) {
        return Err(rejection((
            RojoMappingField::GeneratedRoot,
            None,
            RojoMappingProblem::InvalidRelativePath,
            destination_path,
        )));
    }
    let mut root = RojoTreeNode::with_class("DataModel");
    for generated_module in compiled_project.generated_modules() {
        if let Err(problem) = add_generated_module((&mut root, generated_module, generated_root)) {
            return Err(rejection((
                RojoMappingField::ModuleOutputPath,
                Some(generated_module.module_identity().clone()),
                problem,
                destination_path,
            )));
        }
    }
    let mut json = String::new();
    json.push_str("{\n  \"name\":\"");
    append_json_string(&mut json, project_name);
    json.push_str("\",\n  \"tree\":");
    write_tree_node((&mut json, &root, 1));
    json.push_str("\n}\n");
    if let Err(error) = fs::write(destination_path, json) {
        return Err(rejection((
            RojoMappingField::DestinationPath,
            None,
            RojoMappingProblem::Filesystem(error.kind()),
            destination_path,
        )));
    }
    Ok(())
}

fn valid_generated_root(generated_root: &str) -> bool {
    !generated_root.is_empty()
        && !generated_root.contains('\\')
        && !Path::new(generated_root).is_absolute()
        && generated_root
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn add_generated_module(
    module_parts: (&mut RojoTreeNode, &GeneratedProjectModule, &str),
) -> Result<(), RojoMappingProblem> {
    let (root, generated_module, generated_root) = module_parts;
    let output_path = generated_module.output_path().as_str();
    let mut output_segments = output_path.split('/').peekable();
    let mut current = root;
    while let Some(segment) = output_segments.next() {
        if output_segments.peek().is_none() {
            let instance_name = module_instance_name(segment);
            let child = current
                .children
                .entry(instance_name)
                .or_insert_with(RojoTreeNode::new);
            if child.path.is_some() || !child.children.is_empty() {
                return Err(RojoMappingProblem::DuplicateInstancePath);
            }
            child.path = Some(format!("{generated_root}/{output_path}"));
            return Ok(());
        }
        let child = current
            .children
            .entry(segment.to_owned())
            .or_insert_with(|| RojoTreeNode::with_class(class_for_instance(segment)));
        if child.path.is_some() {
            return Err(RojoMappingProblem::DuplicateInstancePath);
        }
        current = child;
    }
    Err(RojoMappingProblem::InvalidRelativePath)
}

fn module_instance_name(output_file: &str) -> String {
    output_file
        .strip_suffix(".server.luau")
        .or_else(|| output_file.strip_suffix(".client.luau"))
        .or_else(|| output_file.strip_suffix(".luau"))
        .unwrap_or(output_file)
        .to_owned()
}

fn class_for_instance(instance_name: &str) -> &'static str {
    match instance_name {
        "ServerScriptService" => "ServerScriptService",
        "StarterPlayer" => "StarterPlayer",
        "StarterPlayerScripts" => "StarterPlayerScripts",
        "ReplicatedStorage" => "ReplicatedStorage",
        _ => "Folder",
    }
}

struct RojoTreeNode {
    class_name: Option<&'static str>,
    path: Option<String>,
    children: BTreeMap<String, Self>,
}

impl RojoTreeNode {
    const fn new() -> Self {
        Self {
            class_name: None,
            path: None,
            children: BTreeMap::new(),
        }
    }

    fn with_class(class_name: &'static str) -> Self {
        Self {
            class_name: Some(class_name),
            ..Self::new()
        }
    }
}

fn write_tree_node(node_parts: (&mut String, &RojoTreeNode, usize)) {
    let (output, node, indent) = node_parts;
    output.push_str("{\n");
    let mut fields_written = 0;
    if let Some(class_name) = node.class_name {
        write_tree_field((output, indent + 1, "$class", class_name));
        fields_written += 1;
    }
    if let Some(path) = &node.path {
        write_tree_field((output, indent + 1, "$path", path));
        fields_written += 1;
    }
    for (child_name, child) in &node.children {
        if fields_written > 0 {
            output.push_str(",\n");
        }
        write_indent((output, indent + 1));
        output.push('"');
        append_json_string(output, child_name);
        output.push_str("\":");
        write_tree_node((output, child, indent + 1));
        fields_written += 1;
    }
    output.push('\n');
    write_indent((output, indent));
    output.push('}');
}

fn write_tree_field(field_parts: (&mut String, usize, &str, &str)) {
    let (output, indent, field_name, field_value) = field_parts;
    if !output.ends_with("{\n") {
        output.push_str(",\n");
    }
    write_indent((output, indent));
    output.push('"');
    append_json_string(output, field_name);
    output.push_str("\":\"");
    append_json_string(output, field_value);
    output.push('"');
}

fn write_indent(indent_parts: (&mut String, usize)) {
    let (output, indent) = indent_parts;
    for _ in 0..indent {
        output.push_str("  ");
    }
}

fn append_json_string(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
}

fn rejection(
    rejection_parts: (
        RojoMappingField,
        Option<ProjectModuleIdentity>,
        RojoMappingProblem,
        &Path,
    ),
) -> RojoMappingRejection {
    let (field, module_identity, problem, destination_path) = rejection_parts;
    RojoMappingRejection::from_parts((
        field,
        module_identity,
        problem,
        Some(PathBuf::from(destination_path)),
    ))
}
