import { useEffect, useState } from "react";
import "@mantine/core/styles.css";
import {
    ColorSchemeScript,
    Container,
    MantineProvider,
    Stack,
} from "@mantine/core";
import {
    AppToOrchestratorMessage,
    OrchestratorToAppMessage,
    TestInfo,
    TestResult,
} from "./worker_api.tsx";
import { TestAndResult, TestTable } from "./table.tsx";
import { MenuBar } from "./menu.tsx";
import { ProgressBar } from "./progress.tsx";
import { TestVisibilityFilter } from "./filters.tsx";

interface Orchestrator extends Omit<Worker, "postMessage"> {
    postMessage(command: AppToOrchestratorMessage): void;
}

export function App() {
    const [knownTests, setKnownTests] = useState([] as TestInfo[]);
    const [testResults, setTestResults] = useState(
        {} as Record<string, TestResult>,
    );
    const [orchestrator, setOrchestrator] = useState<Orchestrator | null>(null);
    const [numPassedTests, setNumPassedTests] = useState(0);
    const [numFailedTests, setNumFailedTests] = useState(0);
    const [numSkippedTests, setNumSkippedTests] = useState(0);
    const [numQueuedTests, setNumQueuedTests] = useState(0);
    const [visibilityFilter, setVisibilityFilter] =
        useState<TestVisibilityFilter>((): TestVisibilityFilter => {
            return {
                state: {
                    failed: true,
                    running: true,
                    skipped: false,
                    success: false,
                    pending: true,
                },
                renderer: {
                    no: true,
                    optional: true,
                    required: true,
                },
                search: "",
            };
        });

    useEffect(() => {
        const parsedHash = new URLSearchParams(
            window.location.hash.substring(1),
        );
        const worker = new Worker(
            new URL("./workers/orchestrator.tsx", import.meta.url),
            {
                type: "module",
            },
        );
        worker.onmessage = (e) => {
            const message = e.data as OrchestratorToAppMessage;
            switch (message.type) {
                case "test_list":
                    setKnownTests(message.tests);
                    break;
                case "update_test_result":
                    setTestResults((old) => {
                        const results = { ...old };
                        results[message.name] = message.result;
                        return results;
                    });
                    break;
                case "progress_update":
                    setNumQueuedTests(message.total);
                    setNumPassedTests(message.passed);
                    setNumFailedTests(message.failed);
                    setNumSkippedTests(message.skipped);
                    break;
            }
        };
        worker.postMessage({
            type: "configure_workers",
            amount: parseInt(parsedHash.get("runners") || "", 10) || 5,
        });
        setOrchestrator(worker);
        return () => {
            worker.terminate();
        };
    }, []);

    const startTest = (name: string) => {
        orchestrator!.postMessage({ type: "start_test", names: [name] });
    };

    const items: TestAndResult[] = [];
    const searchLower = visibilityFilter.search.toLowerCase() || "";
    for (const testInfo of knownTests) {
        const result = testResults[testInfo.name];
        const state = result == null ? "pending" : result.state;
        if (
            visibilityFilter.state[state] &&
            visibilityFilter.renderer[testInfo.wants_renderer] &&
            testInfo.name.toLowerCase().indexOf(searchLower) != -1
        ) {
            items.push({
                test: testInfo,
                results: result,
            });
        }
    }

    const startAllTests = () => {
        orchestrator!.postMessage({
            type: "start_test",
            names: items.map((test) => test.test.name),
        });
    };

    return (
        <>
            <ColorSchemeScript defaultColorScheme="auto" />
            <MantineProvider defaultColorScheme="auto">
                <Container h="100%" fluid pt="md" pb="md">
                    <Stack h="100%">
                        <MenuBar
                            setSearch={(search) =>
                                setVisibilityFilter((old) => ({
                                    ...old,
                                    search,
                                }))
                            }
                            startAllTests={startAllTests}
                        />
                        <TestTable
                            tests={items}
                            runTest={startTest}
                            filter={visibilityFilter}
                            setFilter={setVisibilityFilter}
                        />
                        <ProgressBar
                            numPassedTests={numPassedTests}
                            numQueuedTests={numQueuedTests}
                            numFailedTests={numFailedTests}
                            numSkippedTests={numSkippedTests}
                        />
                    </Stack>
                </Container>
            </MantineProvider>
        </>
    );
}
