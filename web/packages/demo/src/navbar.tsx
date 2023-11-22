import ruffleLogo from "/logo.svg";
import {
    ChangeEvent,
    FormEvent,
    useEffect,
    useRef,
    useState,
    DragEvent,
} from "react";
import { BaseLoadOptions } from "ruffle-core";
import { DemoSwf, SampleSelection } from "./navbar/samples.tsx";

declare global {
    interface Navigator {
        /**
         * iPadOS sends a User-Agent string that appears to be from macOS.
         * navigator.standalone is not defined on macOS, so we use it for iPad detection.
         */
        standalone?: boolean;
    }
}

interface NavbarProps {
    allowUrlLoading: boolean;
    allowSampleSwfs: boolean;
    onToggleMetadata: () => void;
    onReloadMovie: () => void;
    onSelectUrl: (url: string, options: BaseLoadOptions) => void;
    onSelectFile: (file: File) => void;
    selectedFilename: string | null;
    setSelectedFilename: (value: string | null) => void;
    onFileDragLeave: (event: DragEvent<HTMLElement>) => void;
    onFileDragOver: (event: DragEvent<HTMLElement>) => void;
    onFileDragDrop: (event: DragEvent<HTMLElement>) => void;
}

export function Navbar({
    allowUrlLoading,
    allowSampleSwfs,
    onToggleMetadata,
    onReloadMovie,
    onSelectUrl,
    onSelectFile,
    selectedFilename,
    setSelectedFilename,
    onFileDragLeave,
    onFileDragOver,
    onFileDragDrop,
}: NavbarProps) {
    const localFileInput = useRef<HTMLInputElement>(null);
    const urlInput = useRef<HTMLInputElement>(null);
    const sampleSelectionInput = useRef<HTMLSelectElement>(null);
    const [selectedSample, setSelectedSample] = useState<DemoSwf | null>(null);

    const openFileBrowser = () => {
        localFileInput.current?.click();
    };

    const loadUrl = (url: string) => {
        onSelectUrl(url, {});
        setSelectedFilename(null);
        setSelectedSample(null);
        if (sampleSelectionInput.current) {
            sampleSelectionInput.current.selectedIndex = -1;
        }
    };

    const loadFile = (file: File) => {
        onSelectFile(file);
        setSelectedSample(null);
        setSelectedFilename(file.name);
        if (sampleSelectionInput.current) {
            sampleSelectionInput.current.selectedIndex = -1;
        }
    };

    const submitUrlForm = (e: FormEvent) => {
        e.preventDefault();
        if (urlInput.current?.value) {
            loadUrl(urlInput.current.value);
        }
    };

    const loadSelectedFile = (e: ChangeEvent) => {
        const eventTarget = e.target as HTMLInputElement;
        if (
            eventTarget?.files &&
            eventTarget?.files.length > 0 &&
            eventTarget.files[0]
        ) {
            loadFile(eventTarget.files[0]);
        }
    };

    const confirmAndReload = () => {
        const confirmReload = confirm("Reload the current SWF?");
        if (confirmReload) {
            onReloadMovie();
        }
    };

    const iosInputWorkaround =
        navigator.userAgent.match(/iPad/i) ||
        navigator.userAgent.match(/iPhone/i) ||
        (navigator.platform === "MacIntel" &&
            typeof navigator.standalone !== "undefined");

    useEffect(() => {
        if (selectedFilename != null) {
            setSelectedSample(null);
            if (sampleSelectionInput.current) {
                sampleSelectionInput.current.selectedIndex = -1;
            }
        }
    }, [selectedFilename]);

    return (
        <div id="nav">
            <a id="logo-container" href="https://ruffle.rs/" target="_blank">
                <img className="logo" src={ruffleLogo} alt="Ruffle" />
            </a>
            <div className="select-container">
                <form
                    id="web-url-container"
                    onSubmit={submitUrlForm}
                    hidden={!allowUrlLoading}
                >
                    <input
                        id="web-url"
                        name="web-url"
                        type="text"
                        placeholder="URL of a .swf file on the web"
                        ref={urlInput}
                    />
                    <button id="web-form-submit" type="submit">
                        Load
                    </button>
                </form>
                <div
                    id="local-file-container"
                    onDragLeave={onFileDragLeave}
                    onDragOver={onFileDragOver}
                    onDrop={onFileDragDrop}
                >
                    <span
                        id="local-file-static-label"
                        onClick={openFileBrowser}
                    >
                        Local SWF:
                    </span>
                    <input
                        type="file"
                        accept={iosInputWorkaround ? undefined : ".swf,.spl"}
                        id="local-file"
                        aria-describedby="local-file-static-label"
                        ref={localFileInput}
                        onChange={loadSelectedFile}
                    />
                    <button id="local-file-label" onClick={openFileBrowser}>
                        Select File
                    </button>
                    <label id="local-file-name" htmlFor="local-file">
                        {selectedFilename ?? "No file selected."}
                    </label>
                </div>
                {allowSampleSwfs && (
                    <SampleSelection
                        onSelectUrl={onSelectUrl}
                        selectedSample={selectedSample}
                        setSelectedFilename={setSelectedFilename}
                        setSelectedSample={setSelectedSample}
                        sampleSelectionInput={sampleSelectionInput}
                    />
                )}
            </div>
            <div>
                <svg
                    id="toggle-info"
                    width="20px"
                    viewBox="0 0 416.979 416.979"
                    onClick={onToggleMetadata}
                >
                    <path
                        fill="white"
                        d="M356.004 61.156c-81.37-81.47-213.377-81.551-294.848-.182-81.47 81.371-81.552 213.379-.181 294.85 81.369 81.47 213.378 81.551 294.849.181 81.469-81.369 81.551-213.379.18-294.849zM237.6 340.786a5.821 5.821 0 0 1-5.822 5.822h-46.576a5.821 5.821 0 0 1-5.822-5.822V167.885a5.821 5.821 0 0 1 5.822-5.822h46.576a5.82 5.82 0 0 1 5.822 5.822v172.901zm-29.11-202.885c-18.618 0-33.766-15.146-33.766-33.765 0-18.617 15.147-33.766 33.766-33.766s33.766 15.148 33.766 33.766c0 18.619-15.149 33.765-33.766 33.765z"
                    />
                </svg>
                <svg
                    id="reload-swf"
                    width="20px"
                    viewBox="0 0 489.711 489.711"
                    onClick={confirmAndReload}
                >
                    <path
                        fill="white"
                        d="M112.156 97.111c72.3-65.4 180.5-66.4 253.8-6.7l-58.1 2.2c-7.5.3-13.3 6.5-13 14 .3 7.3 6.3 13 13.5 13h.5l89.2-3.3c7.3-.3 13-6.2 13-13.5v-1.6l-3.3-88.2c-.3-7.5-6.6-13.3-14-13-7.5.3-13.3 6.5-13 14l2.1 55.3c-36.3-29.7-81-46.9-128.8-49.3-59.2-3-116.1 17.3-160 57.1-60.4 54.7-86 137.9-66.8 217.1 1.5 6.2 7 10.3 13.1 10.3 1.1 0 2.1-.1 3.2-.4 7.2-1.8 11.7-9.1 9.9-16.3-16.8-69.6 5.6-142.7 58.7-190.7zm350.3 98.4c-1.8-7.2-9.1-11.7-16.3-9.9-7.2 1.8-11.7 9.1-9.9 16.3 16.9 69.6-5.6 142.7-58.7 190.7-37.3 33.7-84.1 50.3-130.7 50.3-44.5 0-88.9-15.1-124.7-44.9l58.8-5.3c7.4-.7 12.9-7.2 12.2-14.7s-7.2-12.9-14.7-12.2l-88.9 8c-7.4.7-12.9 7.2-12.2 14.7l8 88.9c.6 7 6.5 12.3 13.4 12.3.4 0 .8 0 1.2-.1 7.4-.7 12.9-7.2 12.2-14.7l-4.8-54.1c36.3 29.4 80.8 46.5 128.3 48.9 3.8.2 7.6.3 11.3.3 55.1 0 107.5-20.2 148.7-57.4 60.4-54.7 86-137.8 66.8-217.1z"
                    />
                </svg>
            </div>
        </div>
    );
}
