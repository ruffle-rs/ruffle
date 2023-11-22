import {
    ChangeEvent,
    Fragment,
    RefObject,
    useCallback,
    useEffect,
    useState,
} from "react";
import { BaseLoadOptions } from "ruffle-core";

type SampleCategory = "Animation" | "Game";

const SampleCategories: { [key in SampleCategory]: string } = {
    Animation: "Animations",
    Game: "Games",
};

export interface DemoSwf {
    location: string;
    title?: string;
    author?: string;
    authorLink?: string;
    config?: BaseLoadOptions;
    type: SampleCategory | null;
}

interface SampleSelectionProperties {
    sampleSelectionInput: RefObject<HTMLSelectElement>;
    selectedSample: DemoSwf | null;
    setSelectedSample: (value: DemoSwf | null) => void;
    setSelectedFilename: (name: string | null) => void;
    onSelectUrl: (url: string, config: BaseLoadOptions) => void;
}

export function SampleSelection({
    sampleSelectionInput,
    selectedSample,
    setSelectedSample,
    setSelectedFilename,
    onSelectUrl,
}: SampleSelectionProperties) {
    const [availableSamples, setAvailableSamples] = useState<DemoSwf[]>([]);

    const loadSelectedSample = (e: ChangeEvent) => {
        const eventTarget = e.target as HTMLSelectElement;
        const index = parseInt(eventTarget.value, 10);
        if (availableSamples[index]) {
            loadSample(availableSamples[index]);
        }
    };

    const loadSample = useCallback(
        (swf: DemoSwf) => {
            onSelectUrl(swf.location, swf.config ?? {});
            setSelectedSample(swf);
            setSelectedFilename(null);
        },
        [onSelectUrl, setSelectedFilename, setSelectedSample],
    );

    useEffect(() => {
        (async () => {
            const response = await fetch("swfs.json");

            if (response.ok) {
                const data: { swfs: [DemoSwf] } = await response.json();
                setAvailableSamples(data.swfs);

                if (data.swfs.length > 0) {
                    loadSample(data.swfs[0]);
                }
            }
        })();
    }, [loadSample]);

    return (
        <div
            id="sample-swfs-container"
            className={availableSamples.length == 0 ? "hidden" : ""}
        >
            <span id="sample-swfs-label">Sample SWF:</span>
            <select
                id="sample-swfs"
                aria-describedby="sample-swfs-label"
                onChange={loadSelectedSample}
                ref={sampleSelectionInput}
            >
                {availableSamples.map((sample, i) => (
                    <Fragment key={i}>
                        {sample.type == null && (
                            <option value={i}>{sample.title}</option>
                        )}
                    </Fragment>
                ))}
                {Object.keys(SampleCategories).map((category) => (
                    <optgroup
                        key={category}
                        label={SampleCategories[category as SampleCategory]}
                    >
                        {availableSamples.map((sample, i) => (
                            <Fragment key={i}>
                                {sample.type == category && (
                                    <option value={i}>{sample.title}</option>
                                )}
                            </Fragment>
                        ))}
                    </optgroup>
                ))}
            </select>
            <div
                id="author-container"
                className={selectedSample?.author ? "" : "hidden"}
            >
                <span>Author: </span>
                <a
                    href={selectedSample?.authorLink}
                    target="_blank"
                    id="author"
                >
                    {selectedSample?.author}
                </a>
            </div>
        </div>
    );
}
