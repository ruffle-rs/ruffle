import { RendererRequirement, TestState } from "./worker_api.tsx";
import React from "react";
import { Checkbox, Popover, UnstyledButton } from "@mantine/core";
import classes from "./filter.module.css";
import { IconFilter } from "@tabler/icons-react";

export type TestStateFilter = {
    [state in TestState | "pending"]: boolean;
};

export type RendererRequirementFilter = {
    [state in RendererRequirement]: boolean;
};

export interface TestVisibilityFilter {
    state: TestStateFilter;
    renderer: RendererRequirementFilter;
    search: string;
}

export interface FilterCheckboxProps {
    filter: TestVisibilityFilter;
    setFilter: (filter: TestVisibilityFilter) => void;
}

function FilterCheckbox({
    checked,
    setChecked,
    name,
}: {
    checked: boolean;
    setChecked: (checked: boolean) => void;
    name: string;
}) {
    return (
        <Checkbox
            key={name}
            classNames={{ root: classes.button }}
            label={name}
            checked={checked}
            onChange={(event) => setChecked(event.currentTarget.checked)}
            wrapperProps={{
                onClick: () => setChecked(!checked),
            }}
        />
    );
}

export function StateFilterCheckboxes({
    filter,
    setFilter,
}: FilterCheckboxProps) {
    const names: { [state in TestState | "pending"]: string } = {
        skipped: "Skipped",
        running: "Running",
        failed: "Failed",
        success: "Success",
        pending: "Not Yet Ran",
    };
    return (
        <>
            {Object.entries(names).map(([key, label]) => (
                <FilterCheckbox
                    checked={filter.state[key as TestState]}
                    setChecked={(visible) => {
                        const newFilter: TestVisibilityFilter = {
                            ...filter,
                        };
                        newFilter.state[key as TestState] = visible;
                        setFilter(newFilter);
                    }}
                    name={label}
                />
            ))}
        </>
    );
}

export function RenderFilterCheckboxes({
    filter,
    setFilter,
}: FilterCheckboxProps) {
    const names: { [requirement in RendererRequirement]: string } = {
        no: "Renderer Not Used",
        optional: "Rendering Optional",
        required: "Rendering Required",
    };
    return (
        <>
            {Object.entries(names).map(([key, label]) => (
                <FilterCheckbox
                    checked={filter.renderer[key as RendererRequirement]}
                    setChecked={(visible) => {
                        const newFilter: TestVisibilityFilter = {
                            ...filter,
                        };
                        newFilter.renderer[key as RendererRequirement] =
                            visible;
                        setFilter(newFilter);
                    }}
                    name={label}
                />
            ))}
        </>
    );
}

export function FilterButton({
    filter,
    setFilter,
    Checkboxes,
}: {
    filter: TestVisibilityFilter;
    setFilter: (filter: TestVisibilityFilter) => void;
    Checkboxes: (props: FilterCheckboxProps) => React.JSX.Element;
}) {
    return (
        <Popover width={300} position="bottom" withArrow shadow="md">
            <Popover.Target>
                <UnstyledButton>
                    <IconFilter size="1em" />
                </UnstyledButton>
            </Popover.Target>
            <Popover.Dropdown>
                <Checkboxes setFilter={setFilter} filter={filter} />
            </Popover.Dropdown>
        </Popover>
    );
}
