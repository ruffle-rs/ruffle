import { RendererRequirement, TestInfo, TestResult } from "./worker_api.tsx";
import {
    TableComponents as VirtuosoTableComponents,
    TableVirtuoso,
} from "react-virtuoso";
import {
    Button,
    Code,
    Loader,
    Table,
    TableScrollContainer,
    TableTbody,
    TableTh,
    TableThead,
    TableTr,
    Text,
    Tooltip,
} from "@mantine/core";
import React from "react";
import Ansi from "@cocalc/ansi-to-react";
import classes from "./table.module.css";
import {
    IconCircleCheck,
    IconCircleCheckFilled,
    IconX,
} from "@tabler/icons-react";
import {
    FilterButton,
    RenderFilterCheckboxes,
    StateFilterCheckboxes,
    TestVisibilityFilter,
} from "./filters.tsx";

export interface TestAndResult {
    test: TestInfo;
    results?: TestResult;
}

const TableComponents: VirtuosoTableComponents<TestAndResult> = {
    Scroller: React.forwardRef((props, ref) => (
        <TableScrollContainer {...props} ref={ref} minWidth="lg" />
    )),
    Table: (props) => <Table layout="fixed" {...props} />,
    TableHead: React.forwardRef((props, ref) => (
        <TableThead {...props} ref={ref} />
    )),
    TableRow: (props) => <TableTr {...props} />,
    TableBody: React.forwardRef((props, ref) => (
        <TableTbody {...props} ref={ref} />
    )),
};

function TestResults({ result }: { result: TestResult | null }) {
    if (result?.state == "failed") {
        return (
            <Code block c="red">
                <Ansi>{result?.result}</Ansi>
                {result?.stack}
            </Code>
        );
    }
    if (result?.state == "skipped") {
        return <Text c="gray">Could not run</Text>;
    }
    if (result?.state == "success") {
        return <Text c="green">Success!</Text>;
    }

    return <Loader />;
}

function RendererRequirementIcon({
    requirement,
}: {
    requirement: RendererRequirement;
}) {
    let Icon;
    let tooltip;
    switch (requirement) {
        case "optional":
            Icon = IconCircleCheck;
            tooltip = "Optional";
            break;
        case "required":
            Icon = IconCircleCheckFilled;
            tooltip = "Required";
            break;
        case "no":
        default:
            Icon = IconX;
            tooltip = "Not Used";
            break;
    }
    return (
        <Tooltip label={tooltip}>
            <Icon />
        </Tooltip>
    );
}

export function TestTable({
    tests,
    runTest,
    filter,
    setFilter,
}: {
    tests: TestAndResult[];
    runTest: (name: string) => void;
    filter: TestVisibilityFilter;
    setFilter: (filter: TestVisibilityFilter) => void;
}) {
    return (
        <>
            <TableVirtuoso
                style={{ height: "100%", width: "100%", flex: 1 }}
                data={tests}
                components={TableComponents}
                fixedHeaderContent={() => (
                    <TableTr className={classes.header}>
                        <TableTh w="50%">Name</TableTh>
                        <TableTh w="100">
                            Renderer
                            <FilterButton
                                Checkboxes={RenderFilterCheckboxes}
                                setFilter={setFilter}
                                filter={filter}
                            />
                        </TableTh>
                        <TableTh w="50%">
                            Result
                            <FilterButton
                                Checkboxes={StateFilterCheckboxes}
                                setFilter={setFilter}
                                filter={filter}
                            />
                        </TableTh>
                        <TableTh w="100">Actions</TableTh>
                    </TableTr>
                )}
                itemContent={(_index, test: TestAndResult) => (
                    <>
                        <Table.Td>{test.test.name}</Table.Td>
                        <Table.Td>
                            <RendererRequirementIcon
                                requirement={test.test.wants_renderer}
                            />
                        </Table.Td>
                        <Table.Td>
                            {test.results && (
                                <TestResults result={test.results} />
                            )}
                        </Table.Td>
                        <Table.Td>
                            <Button
                                onClick={() => runTest(test.test.name)}
                                disabled={!test.test.should_run}
                            >
                                Run
                            </Button>
                        </Table.Td>
                    </>
                )}
            />
        </>
    );
}
