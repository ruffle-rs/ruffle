export type RendererRequirement = "no" | "optional" | "required";

export interface TestInfo {
    ignored: boolean;
    known_failure: boolean;
    name: string;
    should_run: boolean;
    wants_renderer: RendererRequirement;
}

export interface ConfigureWorkers {
    type: "configure_workers";
    amount: number;
}

export type TestState = "skipped" | "running" | "failed" | "success";

export interface TestResult {
    result?: string;
    stack?: string;
    state: TestState;
}

export interface TestList {
    type: "test_list";
    tests: TestInfo[];
}

export interface UpdateTestResult {
    type: "update_test_result";
    name: string;
    result: TestResult;
}

export interface ProgressUpdate {
    type: "progress_update";
    total: number;
    done: number;
    passed: number;
    failed: number;
    skipped: number;
}

export type OrchestratorToAppMessage =
    | TestList
    | UpdateTestResult
    | ProgressUpdate;

export interface StartTest {
    type: "start_test";
    names: string[];
}

export type AppToOrchestratorMessage = StartTest | ConfigureWorkers;

export interface TestFinished {
    type: "test_finished";
    result: TestResult;
    needsRestart: boolean;
}

export type RunnerToOrchestratorMessage = TestFinished;
