import { Progress } from "@mantine/core";

export function ProgressBar({
    numPassedTests,
    numQueuedTests,
    numFailedTests,
    numSkippedTests,
}: {
    numPassedTests: number;
    numQueuedTests: number;
    numFailedTests: number;
    numSkippedTests: number;
}) {
    return (
        <Progress.Root size="xl">
            <Progress.Section
                value={(numPassedTests / numQueuedTests) * 100}
                color="green"
                title="Passed"
            >
                <Progress.Label>{numPassedTests}</Progress.Label>
            </Progress.Section>
            <Progress.Section
                value={(numFailedTests / numQueuedTests) * 100}
                color="pink"
                title="Failed"
            >
                <Progress.Label>{numFailedTests}</Progress.Label>
            </Progress.Section>
            <Progress.Section
                value={(numSkippedTests / numQueuedTests) * 100}
                color="gray"
                title="Skipped"
            >
                <Progress.Label>{numSkippedTests}</Progress.Label>
            </Progress.Section>
        </Progress.Root>
    );
}
