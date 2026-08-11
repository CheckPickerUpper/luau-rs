//! Command parsing and project compilation orchestration for the `luau-rs` executable.

use std::{
    env, fmt, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
};

use roblox_rust::{
    compile_project, write_compiled_project_atomically, CompiledProject, ProjectCompilationOutcome,
    ProjectCompilationProblem, ProjectCompilationRejection, ProjectCompilationRequest,
    ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource, ProjectOutputRejection,
};

const HELP_TEXT: &str = "luau-rs check <project-file>\nluau-rs compile <project-file> --output <directory>\n\nProject files contain one module per line:\n  <server|client|shared> <entrypoint|library> <module-path> <source-file>\nBlank lines and lines beginning with # are ignored. Source paths are relative to the project file.\n";

pub fn run() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match parse_command(&arguments).and_then(run_command) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            process::exit(error.exit_code());
        }
    }
}

enum Command {
    Help,
    Check {
        project_file: PathBuf,
    },
    Compile {
        project_file: PathBuf,
        output_directory: PathBuf,
    },
}

fn parse_command(arguments: &[String]) -> Result<Command, CliError> {
    let Some(command_name) = arguments.first() else {
        return Ok(Command::Help);
    };
    if command_name == "help" || command_name == "--help" || command_name == "-h" {
        return Ok(Command::Help);
    }
    match command_name.as_str() {
        "check" => parse_check_command(arguments),
        "compile" => parse_compile_command(arguments),
        _ => Err(CliError::Usage(format!("unknown command `{command_name}`"))),
    }
}

fn parse_check_command(arguments: &[String]) -> Result<Command, CliError> {
    if arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        return Ok(Command::Help);
    }
    let values = arguments[1..].iter().collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(CliError::Usage(
            "check requires exactly one project file".to_owned(),
        ));
    }
    Ok(Command::Check {
        project_file: PathBuf::from(values[0]),
    })
}

fn parse_compile_command(arguments: &[String]) -> Result<Command, CliError> {
    let mut project_file = None;
    let mut output_directory = None;
    let mut argument_index = 1;
    while argument_index < arguments.len() {
        let argument = &arguments[argument_index];
        if argument == "--help" || argument == "-h" {
            return Ok(Command::Help);
        }
        if argument == "--output" {
            if output_directory.is_some() {
                return Err(CliError::Usage(
                    "compile accepts --output only once".to_owned(),
                ));
            }
            argument_index += 1;
            let Some(output) = arguments.get(argument_index) else {
                return Err(CliError::Usage(
                    "compile requires a directory after --output".to_owned(),
                ));
            };
            output_directory = Some(PathBuf::from(output));
        } else if argument.starts_with('-') {
            return Err(CliError::Usage(format!(
                "unknown compile option `{argument}`"
            )));
        } else if project_file.is_some() {
            return Err(CliError::Usage(
                "compile accepts one project file".to_owned(),
            ));
        } else {
            project_file = Some(PathBuf::from(argument));
        }
        argument_index += 1;
    }
    let Some(project_file) = project_file else {
        return Err(CliError::Usage(
            "compile requires one project file".to_owned(),
        ));
    };
    let Some(output_directory) = output_directory else {
        return Err(CliError::Usage(
            "compile requires --output <directory>".to_owned(),
        ));
    };
    Ok(Command::Compile {
        project_file,
        output_directory,
    })
}

fn run_command(command: Command) -> Result<String, CliError> {
    match command {
        Command::Help => Ok(HELP_TEXT.to_owned()),
        Command::Check { project_file } => {
            let project = LoadedProject::from_file(&project_file)?;
            match compile_project(project.request()) {
                ProjectCompilationOutcome::Compiled(_) => {
                    Ok(format!("checked {}", project_file.display()))
                }
                ProjectCompilationOutcome::Rejected(rejection) => {
                    Err(project.compilation_error(&rejection))
                }
            }
        }
        Command::Compile {
            project_file,
            output_directory,
        } => {
            let project = LoadedProject::from_file(&project_file)?;
            let compiled_project = match compile_project(project.request()) {
                ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
                ProjectCompilationOutcome::Rejected(rejection) => {
                    return Err(project.compilation_error(&rejection));
                }
            };
            publish_project((&compiled_project, &output_directory))?;
            Ok(format!(
                "compiled {} -> {}",
                project_file.display(),
                output_directory.display()
            ))
        }
    }
}

fn publish_project(project_parts: (&CompiledProject, &Path)) -> Result<(), CliError> {
    write_compiled_project_atomically(project_parts)
        .map_err(|rejection| CliError::from_output_rejection(&rejection))
}

struct LoadedProject {
    project_file: PathBuf,
    modules: Vec<LoadedModule>,
}

struct LoadedModule {
    identity: ProjectModuleIdentity,
    role: ProjectModuleRole,
    source_path: PathBuf,
    source_text: String,
}

impl LoadedProject {
    fn from_file(project_file: &Path) -> Result<Self, CliError> {
        let project_text = fs::read_to_string(project_file).map_err(|error| {
            CliError::input((
                project_file.to_owned(),
                None,
                format!("cannot read project file: {:?}", error.kind()),
            ))
        })?;
        let project_parent = project_file.parent().unwrap_or_else(|| Path::new("."));
        let mut modules = Vec::new();
        for (line_index, line) in project_text.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }
            let fields = trimmed_line.split_whitespace().collect::<Vec<_>>();
            if fields.len() != 4 {
                return Err(CliError::input((
                    project_file.to_owned(),
                    Some(line_number),
                    "module lines require side, role, module path, and source file".to_owned(),
                )));
            }
            let identity = parse_identity((fields[0], fields[2])).map_err(|message| {
                CliError::input((project_file.to_owned(), Some(line_number), message))
            })?;
            let role = parse_module_role(fields[1]).map_err(|message| {
                CliError::input((project_file.to_owned(), Some(line_number), message))
            })?;
            let source_path = project_parent.join(fields[3]);
            let source_text = fs::read_to_string(&source_path).map_err(|error| {
                CliError::input((
                    source_path.clone(),
                    Some(line_number),
                    format!("cannot read source file: {:?}", error.kind()),
                ))
            })?;
            modules.push(LoadedModule {
                identity,
                role,
                source_path,
                source_text,
            });
        }
        Ok(Self {
            project_file: project_file.to_owned(),
            modules,
        })
    }

    fn request(&self) -> ProjectCompilationRequest {
        ProjectCompilationRequest::from_source_modules(
            self.modules
                .iter()
                .map(|module| {
                    ProjectModuleSource::from_source_parts((
                        module.identity.clone(),
                        module.role,
                        module.source_text.clone(),
                    ))
                })
                .collect(),
        )
    }

    fn compilation_error(&self, rejection: &ProjectCompilationRejection) -> CliError {
        let file_path = self.file_for_problem(rejection.first_problem());
        CliError::Compilation {
            file_path,
            detail: format!("{:?}", rejection.first_problem()),
        }
    }

    fn file_for_problem(&self, problem: &ProjectCompilationProblem) -> PathBuf {
        let module_identity = match problem {
            ProjectCompilationProblem::SharedModuleCannotBeEntrypoint { module_identity }
            | ProjectCompilationProblem::InvalidModuleIdentity { module_identity }
            | ProjectCompilationProblem::DuplicateModuleIdentity { module_identity }
            | ProjectCompilationProblem::SourceModuleRejected {
                module_identity, ..
            }
            | ProjectCompilationProblem::ImportedModuleNotFound {
                importing_module_identity: module_identity,
                ..
            }
            | ProjectCompilationProblem::ImportedModuleIsEntrypoint {
                importing_module_identity: module_identity,
                ..
            }
            | ProjectCompilationProblem::ImportExecutionSideNotAllowed {
                importing_module_identity: module_identity,
                ..
            }
            | ProjectCompilationProblem::ImportedFunctionNotFound {
                importing_module_identity: module_identity,
                ..
            }
            | ProjectCompilationProblem::ImportedFunctionIsPrivate {
                importing_module_identity: module_identity,
                ..
            }
            | ProjectCompilationProblem::ImportNameCollidesWithLocalDeclaration {
                importing_module_identity: module_identity,
                ..
            } => Some(module_identity),
            ProjectCompilationProblem::ImportCycle { cycle_path } => cycle_path.first(),
            ProjectCompilationProblem::MissingEntrypointModule => None,
        };
        let Some(module_identity) = module_identity else {
            return self.project_file.clone();
        };
        self.modules
            .iter()
            .find(|module| module.identity == *module_identity)
            .map_or_else(
                || self.project_file.clone(),
                |module| module.source_path.clone(),
            )
    }
}

fn parse_identity(identity_parts: (&str, &str)) -> Result<ProjectModuleIdentity, String> {
    let (side_name, module_path) = identity_parts;
    if module_path.is_empty() {
        return Err("module path cannot be empty".to_owned());
    }
    match side_name {
        "server" => Ok(ProjectModuleIdentity::Server {
            module_path: module_path.to_owned(),
        }),
        "client" => Ok(ProjectModuleIdentity::Client {
            module_path: module_path.to_owned(),
        }),
        "shared" => Ok(ProjectModuleIdentity::Shared {
            module_path: module_path.to_owned(),
        }),
        _ => Err(format!(
            "unknown execution side `{side_name}`; expected server, client, or shared"
        )),
    }
}

fn parse_module_role(role_name: &str) -> Result<ProjectModuleRole, String> {
    match role_name {
        "entrypoint" => Ok(ProjectModuleRole::Entrypoint),
        "library" => Ok(ProjectModuleRole::Library),
        _ => Err(format!(
            "unknown module role `{role_name}`; expected entrypoint or library"
        )),
    }
}

enum CliError {
    Usage(String),
    Input {
        path: PathBuf,
        line: Option<usize>,
        message: String,
    },
    Compilation {
        file_path: PathBuf,
        detail: String,
    },
    Output {
        path: PathBuf,
        operation: String,
        error_kind: ErrorKind,
    },
}

impl CliError {
    fn input(input_parts: (PathBuf, Option<usize>, String)) -> Self {
        let (path, line, message) = input_parts;
        Self::Input {
            path,
            line,
            message,
        }
    }

    fn from_output_rejection(rejection: &ProjectOutputRejection) -> Self {
        Self::Output {
            path: rejection.output_path().to_owned(),
            operation: format!("{:?}", rejection.operation()),
            error_kind: rejection.error_kind(),
        }
    }

    const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Input { .. } | Self::Compilation { .. } | Self::Output { .. } => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "error: {message}\n\n{HELP_TEXT}"),
            Self::Input {
                path,
                line,
                message,
            } => match line {
                Some(line) => write!(formatter, "{}:{line}: error: {message}", path.display()),
                None => write!(formatter, "{}: error: {message}", path.display()),
            },
            Self::Compilation { file_path, detail } => {
                write!(formatter, "{}: error: {detail}", file_path.display())
            }
            Self::Output {
                path,
                operation,
                error_kind,
            } => write!(
                formatter,
                "{}: output {operation} failed: {error_kind:?}",
                path.display()
            ),
        }
    }
}
