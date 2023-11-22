import { Player } from "./player.tsx";
import { useRef, useState, DragEvent, useCallback } from "react";
import { BaseLoadOptions, MovieMetadata } from "ruffle-core";
import { Navbar } from "./navbar.tsx";
import { MetadataPanel } from "./metadata.tsx";

interface AppProperties {
    ruffleBaseConfig: BaseLoadOptions;
    allowUrlLoading: boolean;
    allowSampleSwfs: boolean;
}

export function App({
    ruffleBaseConfig,
    allowUrlLoading,
    allowSampleSwfs,
}: AppProperties) {
    const [metadata, setMetadata] = useState<MovieMetadata | null>(null);
    const [metadataVisible, setMetadataVisible] = useState<boolean>(false);
    const [selectedFilename, setSelectedFilename] = useState<string | null>(
        null,
    );
    const [dragOverlayVisible, setDragOverlayVisible] =
        useState<boolean>(false);
    const player = useRef<Player>(null);

    const toggleMetadataVisible = () => {
        setMetadataVisible(!metadataVisible);
    };

    const reloadMovie = () => {
        player.current?.reload();
    };

    // useCallback because this will be called from useEffect, we need this function to not change
    const onSelectUrl = useCallback((url: string, options: BaseLoadOptions) => {
        player.current?.loadUrl(url, options);
    }, []);

    const onSelectFile = (file: File) => {
        player.current?.loadFile(file);
    };

    const onFileDragEnter = (event: DragEvent<HTMLElement>) => {
        event.stopPropagation();
        event.preventDefault();
    };

    const onFileDragLeave = (event: DragEvent<HTMLElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setDragOverlayVisible(false);
    };

    const onFileDragOver = (event: DragEvent<HTMLElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setDragOverlayVisible(true);
    };

    const onFileDragDrop = (event: DragEvent<HTMLElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setDragOverlayVisible(false);
        if (event.dataTransfer) {
            setSelectedFilename(event.dataTransfer.files[0].name);
            player.current?.loadFile(event.dataTransfer.files[0]);
        }
    };

    return (
        <>
            <Navbar
                allowUrlLoading={allowUrlLoading}
                allowSampleSwfs={allowSampleSwfs}
                onToggleMetadata={toggleMetadataVisible}
                onReloadMovie={reloadMovie}
                onSelectUrl={onSelectUrl}
                onSelectFile={onSelectFile}
                selectedFilename={selectedFilename}
                setSelectedFilename={setSelectedFilename}
                onFileDragLeave={onFileDragLeave}
                onFileDragOver={onFileDragOver}
                onFileDragDrop={onFileDragDrop}
            />

            <div
                id="main"
                className={metadataVisible ? "info-container-shown" : ""}
            >
                <Player
                    id="player-container"
                    aria-label="Select a demo or drag an SWF"
                    onLoadedMetadata={setMetadata}
                    ref={player}
                    onDragEnter={onFileDragEnter}
                    onDragLeave={onFileDragLeave}
                    onDragOver={onFileDragOver}
                    onDragDrop={onFileDragDrop}
                    baseConfig={ruffleBaseConfig}
                >
                    <div
                        id="overlay"
                        className={dragOverlayVisible ? "drag" : ""}
                    ></div>
                </Player>
                <MetadataPanel visible={metadataVisible} metadata={metadata} />
            </div>
        </>
    );
}
