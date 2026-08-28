//! Bounded official WebAssembly core-suite contracts for the compiler pipeline.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::rstest;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Error;
use std::process::Command;
use support::official_luau_tool;
use wast::core::{WastArgCore, WastRetCore};
use wast::parser::{self, ParseBuffer};
use wast::{QuoteWat, Wast, WastArg, WastDirective, WastExecute, WastRet};

const SPEC_FIXTURE: &str = include_str!("fixtures/wasm-spec/core/i32-arithmetic.wast");
const MINIMUM_PASSED_ASSERTIONS: usize = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AssertionId(usize);

#[derive(Debug, Clone, Copy)]
enum AssertionExpectation {
    Return(i32),
    Trap,
}

#[derive(Debug)]
struct SpecAssertion {
    id: AssertionId,
    line: usize,
    export_name: String,
    arguments: Vec<i32>,
    expectation: AssertionExpectation,
}

#[derive(Debug)]
enum ModuleCompilation {
    Ready(String),
    Rejected(String),
}

#[derive(Debug)]
struct ModuleCase {
    compilation: ModuleCompilation,
    assertions: Vec<SpecAssertion>,
}

#[derive(Debug, Default)]
struct SuiteReport {
    passed: usize,
    failures: Vec<String>,
    skipped_by_scope: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
enum HarnessError {
    #[error("unsupported fixture construct: {0}")]
    Unsupported(&'static str),
}

impl SuiteReport {
    const fn record_pass(&mut self) {
        self.passed += 1;
    }

    fn record_failure(&mut self, detail: impl Into<String>) {
        self.failures.push(detail.into());
    }

    fn record_skip(&mut self, detail: impl Into<String>) {
        self.skipped_by_scope.push(detail.into());
    }

    fn summary(&self) -> String {
        format!(
            "official wasm core subset: passed={}, failed={}, skipped-by-scope={}",
            self.passed,
            self.failures.len(),
            self.skipped_by_scope.len()
        )
    }

    fn rendered(&self) -> String {
        let mut report = self.summary();
        for failure in &self.failures {
            let _ = write!(report, "\nFAIL: {failure}");
        }
        for skipped in &self.skipped_by_scope {
            let _ = write!(report, "\nSKIP: {skipped}");
        }
        report
    }
}

#[rstest]
fn given_official_i32_core_assertions_when_run_through_luau_then_they_pass() -> Result<(), Error> {
    let report = run_spec_suite();
    tracing::info!(
        passed = report.passed,
        failed = report.failures.len(),
        skipped_by_scope = report.skipped_by_scope.len(),
        failures = ?report.failures,
        skipped = ?report.skipped_by_scope,
        "official WebAssembly core-suite result"
    );
    if !report.failures.is_empty() {
        return Err(Error::other(report.rendered()));
    }
    if report.passed == MINIMUM_PASSED_ASSERTIONS {
        Ok(())
    } else {
        Err(Error::other(format!(
            "official wasm core pass-count regression: expected {}, observed {}",
            MINIMUM_PASSED_ASSERTIONS, report.passed
        )))
    }
}

fn run_spec_suite() -> SuiteReport {
    let mut report = SuiteReport::default();
    let fixture: &'static str = Box::leak(SPEC_FIXTURE.to_owned().into_boxed_str());
    let buffer: &'static ParseBuffer<'static> = match ParseBuffer::new(fixture) {
        Ok(buffer) => Box::leak(Box::new(buffer)),
        Err(error) => {
            report.record_failure(format!("could not parse official fixture: {error}"));
            return report;
        }
    };
    let mut script = match parser::parse::<Wast>(buffer) {
        Ok(script) => script,
        Err(error) => {
            report.record_failure(format!("could not parse official fixture: {error}"));
            return report;
        }
    };
    let mut current = None;
    let mut next_id = 0;

    for directive in &mut script.directives {
        match directive {
            WastDirective::Module(module) => {
                finish_module(current.take(), &mut report);
                current = Some(ModuleCase {
                    compilation: compile_module(module),
                    assertions: Vec::new(),
                });
            }
            WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                let assertion =
                    match parse_return_assertion(*span, exec, results, AssertionId(next_id)) {
                        Ok(assertion) => {
                            next_id += 1;
                            assertion
                        }
                        Err(reason) => {
                            report.record_skip(format!("line {}: {reason}", line_number(*span)));
                            continue;
                        }
                    };
                match current.as_mut() {
                    Some(module) => module.assertions.push(assertion),
                    None => report.record_skip(format!(
                        "line {}: return assertion has no executable module",
                        line_number(*span)
                    )),
                }
            }
            WastDirective::AssertTrap { span, exec, .. } => {
                let assertion = match parse_trap_assertion(*span, exec, AssertionId(next_id)) {
                    Ok(assertion) => {
                        next_id += 1;
                        assertion
                    }
                    Err(reason) => {
                        report.record_failure(format!(
                            "line {}: could not exercise official trap assertion: {reason}",
                            line_number(*span)
                        ));
                        continue;
                    }
                };
                match current.as_mut() {
                    Some(module) => module.assertions.push(assertion),
                    None => report.record_failure(format!(
                        "line {}: official trap assertion has no executable module",
                        line_number(*span)
                    )),
                }
            }
            WastDirective::AssertExhaustion { span, .. }
            | WastDirective::AssertUnlinkable { span, .. }
            | WastDirective::AssertMalformed { span, .. }
            | WastDirective::AssertMalformedCustom { span, .. }
            | WastDirective::AssertInvalid { span, .. }
            | WastDirective::AssertInvalidCustom { span, .. }
            | WastDirective::AssertException { span, .. }
            | WastDirective::AssertSuspension { span, .. } => {
                report.record_skip(format!(
                    "line {}: assertion kind is outside the executable i32 subset",
                    line_number(*span)
                ));
            }
            WastDirective::ModuleDefinition { .. }
            | WastDirective::ModuleInstance { .. }
            | WastDirective::Register { .. }
            | WastDirective::Invoke(_)
            | WastDirective::Thread(_)
            | WastDirective::Wait { .. } => {}
        }
    }
    finish_module(current, &mut report);
    report
}

fn compile_module(module: &mut QuoteWat<'static>) -> ModuleCompilation {
    let wasm = match module.encode() {
        Ok(wasm) => wasm,
        Err(error) => return ModuleCompilation::Rejected(format!("WAST encoding failed: {error}")),
    };
    let decoded = match decode_module(&wasm) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return ModuleCompilation::Rejected(format!("decoder rejected module: {rejection:?}"));
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => ModuleCompilation::Ready(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => {
            ModuleCompilation::Rejected(format!("translator rejected module: {rejection:?}"))
        }
    }
}

fn parse_return_assertion(
    span: wast::token::Span,
    exec: &mut WastExecute<'static>,
    results: &[WastRet<'static>],
    id: AssertionId,
) -> Result<SpecAssertion, HarnessError> {
    let invoke = parse_invoke(exec)?;
    let arguments = parse_i32_arguments(&invoke.args)?;
    let [WastRet::Core(WastRetCore::I32(expected))] = results else {
        return Err(HarnessError::Unsupported("return values are not i32"));
    };
    Ok(SpecAssertion {
        id,
        line: line_number(span),
        export_name: invoke.name.to_owned(),
        arguments,
        expectation: AssertionExpectation::Return(*expected),
    })
}

fn parse_trap_assertion(
    span: wast::token::Span,
    exec: &mut WastExecute<'static>,
    id: AssertionId,
) -> Result<SpecAssertion, HarnessError> {
    let invoke = parse_invoke(exec)?;
    let arguments = parse_i32_arguments(&invoke.args)?;
    Ok(SpecAssertion {
        id,
        line: line_number(span),
        export_name: invoke.name.to_owned(),
        arguments,
        expectation: AssertionExpectation::Trap,
    })
}

const fn parse_invoke<'a>(
    exec: &'a mut WastExecute<'static>,
) -> Result<&'a mut wast::WastInvoke<'static>, HarnessError> {
    match exec {
        WastExecute::Invoke(invoke) if invoke.module.is_none() => Ok(invoke),
        WastExecute::Invoke(_) => Err(HarnessError::Unsupported("named module invocations")),
        WastExecute::Wat(_) => Err(HarnessError::Unsupported("inline module execution")),
        WastExecute::Get { .. } => Err(HarnessError::Unsupported("global reads")),
    }
}

fn parse_i32_arguments(arguments: &[WastArg<'static>]) -> Result<Vec<i32>, HarnessError> {
    arguments
        .iter()
        .map(|argument| match argument {
            WastArg::Core(WastArgCore::I32(value)) => Ok(*value),
            WastArg::Core(_) => Err(HarnessError::Unsupported("non-i32 invocation arguments")),
            _ => Err(HarnessError::Unsupported("component invocation arguments")),
        })
        .collect()
}

fn finish_module(current: Option<ModuleCase>, report: &mut SuiteReport) {
    let Some(module) = current else {
        return;
    };
    match module.compilation {
        ModuleCompilation::Ready(generated) => {
            run_ready_module(&generated, &module.assertions, report);
        }
        ModuleCompilation::Rejected(reason) => {
            for assertion in module.assertions {
                match assertion.expectation {
                    AssertionExpectation::Return(_) => report.record_skip(format!(
                        "line {}: return assertion skipped because the module is outside compiler scope ({reason})",
                        assertion.line
                    )),
                    AssertionExpectation::Trap => report.record_failure(format!(
                        "line {}: trap assertion could not reach the backend ({reason})",
                        assertion.line
                    )),
                }
            }
        }
    }
}

fn run_ready_module(generated: &str, assertions: &[SpecAssertion], report: &mut SuiteReport) {
    let luau = match official_luau_tool(("LUAU_BIN", "luau")) {
        Ok(path) => path,
        Err(error) => {
            for assertion in assertions {
                report.record_failure(format!(
                    "line {}: pinned Luau runtime unavailable: {error}",
                    assertion.line
                ));
            }
            return;
        }
    };
    let temp_dir = match tempfile::Builder::new()
        .prefix("luau-rs-wasm-spec")
        .tempdir()
    {
        Ok(temp_dir) => temp_dir,
        Err(error) => {
            for assertion in assertions {
                report.record_failure(format!(
                    "line {}: could not create runner directory: {error}",
                    assertion.line
                ));
            }
            return;
        }
    };
    let source_path = temp_dir.path().join("driver.luau");
    let driver = build_driver(generated, assertions);
    if let Err(error) = fs_err::write(&source_path, driver) {
        for assertion in assertions {
            report.record_failure(format!(
                "line {}: could not write runner: {error}",
                assertion.line
            ));
        }
        return;
    }
    let output = match Command::new(luau).arg(&source_path).output() {
        Ok(output) => output,
        Err(error) => {
            for assertion in assertions {
                report.record_failure(format!(
                    "line {}: could not run pinned Luau: {error}",
                    assertion.line
                ));
            }
            return;
        }
    };
    let execution = parse_execution_output(&output.stdout);
    for assertion in assertions {
        match execution.get(&assertion.id) {
            Some(ExecutionResult::Passed) => report.record_pass(),
            Some(ExecutionResult::Failed(detail)) => report.record_failure(format!(
                "line {}: {} ({detail})",
                assertion.line,
                assertion_label(assertion)
            )),
            None => report.record_failure(format!(
                "line {}: {} produced no result; stderr={}",
                assertion.line,
                assertion_label(assertion),
                String::from_utf8_lossy(&output.stderr)
            )),
        }
    }
}

fn build_driver(generated: &str, assertions: &[SpecAssertion]) -> String {
    let mut driver = format!(
        "local function load_module()\n{generated}\nend\nlocal module = load_module()({{}})\n"
    );
    for assertion in assertions {
        let call = format!(
            "module[{export:?}]({arguments})",
            export = assertion.export_name,
            arguments = assertion
                .arguments
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let check = match assertion.expectation {
            AssertionExpectation::Return(expected) => format!(
                "local ok_{id}, actual_{id} = pcall(function() return {call} end)\nif not ok_{id} then\n    print(\"FAIL|{id}|unexpected trap: \" .. tostring(actual_{id}))\nelseif actual_{id} ~= {expected} then\n    print(\"FAIL|{id}|expected {expected} but got \" .. tostring(actual_{id}))\nelse\n    print(\"PASS|{id}\")\nend\n",
                id = assertion.id.0,
                call = call,
                expected = expected
            ),
            AssertionExpectation::Trap => format!(
                "local ok_{id}, actual_{id} = pcall(function() return {call} end)\nif ok_{id} then\n    print(\"FAIL|{id}|expected a trap\")\nelse\n    print(\"PASS|{id}\")\nend\n",
                id = assertion.id.0,
                call = call
            ),
        };
        driver.push_str(&check);
    }
    driver
}

#[derive(Debug)]
enum ExecutionResult {
    Passed,
    Failed(String),
}

fn parse_execution_output(stdout: &[u8]) -> BTreeMap<AssertionId, ExecutionResult> {
    let mut results = BTreeMap::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        let mut fields = line.splitn(3, '|');
        let Some(kind) = fields.next() else { continue };
        let id = match fields.next() {
            Some(field) => match field.parse::<usize>() {
                Ok(id) => AssertionId(id),
                Err(_) => continue,
            },
            None => continue,
        };
        match kind {
            "PASS" => {
                results.insert(id, ExecutionResult::Passed);
            }
            "FAIL" => {
                let detail = fields
                    .next()
                    .map_or_else(|| "unspecified failure".to_owned(), str::to_owned);
                results.insert(id, ExecutionResult::Failed(detail));
            }
            _ => {}
        }
    }
    results
}

fn assertion_label(assertion: &SpecAssertion) -> String {
    match assertion.expectation {
        AssertionExpectation::Return(expected) => {
            format!("invoke {} expected {expected}", assertion.export_name)
        }
        AssertionExpectation::Trap => format!("invoke {} expected trap", assertion.export_name),
    }
}

fn line_number(span: wast::token::Span) -> usize {
    span.linecol_in(SPEC_FIXTURE).0 + 1
}
