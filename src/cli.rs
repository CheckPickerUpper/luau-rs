//! Command parsing and project compilation orchestration for the `luau-rs` executable.

use std::{
    env,
    path::{Path, PathBuf},
    process,
};

use roblox_rust::{
    compile_project, write_compiled_project_atomically, CompiledProject, ProjectCompilationOutcome,
};

mod error;
mod project_file;

use error::CliError;
use project_file::LoadedProject;

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
                    Err(CliError::compilation(rejection.first_diagnostic()))
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
                    return Err(CliError::compilation(rejection.first_diagnostic()));
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
