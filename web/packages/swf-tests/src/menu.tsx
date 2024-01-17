import { useRef } from "react";
import { ActionIcon, Button, Group, rem, TextInput } from "@mantine/core";
import classNames from "./menu.module.css";
import { IconArrowRight, IconSearch } from "@tabler/icons-react";

export function MenuBar({
    setSearch,
    startAllTests,
}: {
    setSearch: (search: string) => void;
    startAllTests: () => void;
}) {
    const searchInput = useRef<HTMLInputElement | null>(null);

    const applySearch = () => {
        setSearch(searchInput.current?.value || "");
    };

    return (
        <Group>
            <TextInput
                className={classNames.search}
                ref={searchInput}
                radius="xl"
                size="md"
                placeholder="Search tests"
                rightSectionWidth={42}
                onChange={applySearch}
                leftSection={
                    <IconSearch
                        style={{
                            width: rem(18),
                            height: rem(18),
                        }}
                        stroke={1.5}
                    />
                }
                rightSection={
                    <ActionIcon
                        size={32}
                        radius="xl"
                        variant="filled"
                        onClick={applySearch}
                    >
                        <IconArrowRight
                            style={{
                                width: rem(18),
                                height: rem(18),
                            }}
                            stroke={1.5}
                        />
                    </ActionIcon>
                }
            />
            <Button onClick={startAllTests}>Run All</Button>
        </Group>
    );
}
