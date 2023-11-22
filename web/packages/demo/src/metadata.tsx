import { MovieMetadata } from "ruffle-core";

interface MetadataProps {
    visible: boolean;
    metadata: MovieMetadata | null;
}

const swfToFlashVersion: { [key: number]: string } = {
    1: "1",
    2: "2",
    3: "3",
    4: "4",
    5: "5",
    6: "6",
    7: "7",
    8: "8",
    9: "9.0",
    10: "10.0/10.1",
    11: "10.2",
    12: "10.3",
    13: "11.0",
    14: "11.1",
    15: "11.2",
    16: "11.3",
    17: "11.4",
    18: "11.5",
    19: "11.6",
    20: "11.7",
    21: "11.8",
    22: "11.9",
    23: "12",
    24: "13",
    25: "14",
    26: "15",
    27: "16",
    28: "17",
    29: "18",
    30: "19",
    31: "20",
    32: "21",
    33: "22",
    34: "23",
    35: "24",
    36: "25",
    37: "26",
    38: "27",
    39: "28",
    40: "29",
    41: "30",
    42: "31",
    43: "32",
};

export function MetadataPanel({ visible, metadata }: MetadataProps) {
    return (
        <div id="info-container" className={visible ? "" : "hidden"}>
            <div>
                <span>Uncompressed Length</span>
                <span id="uncompressedLength">
                    {(metadata?.uncompressedLength ?? 0) >> 10}Kb
                </span>
            </div>
            <div>
                <span>SWF Version</span>
                <span id="swfVersion">{metadata?.swfVersion}</span>
            </div>
            <div>
                <span>FP Version</span>
                <span id="flashVersion">
                    {metadata
                        ? swfToFlashVersion[metadata.swfVersion] ?? "Unknown"
                        : ""}
                </span>
            </div>
            <div>
                <span>ActionScript 3</span>
                <span id="isActionScript3">
                    {metadata?.isActionScript3 ? "true" : "false"}
                </span>
            </div>
            <div>
                <span>Total Frames</span>
                <span id="numFrames">{metadata?.numFrames}</span>
            </div>
            <div>
                <span>Frame Rate</span>
                <span id="frameRate">{metadata?.frameRate}</span>
            </div>
            <div>
                <span>SWF Width</span>
                <span id="width">{metadata?.width}</span>
            </div>
            <div>
                <span>SWF Height</span>
                <span id="height">{metadata?.height}</span>
            </div>
            <div>
                <span>SWF Background Color</span>
                <span
                    id="backgroundColor"
                    style={{
                        backgroundColor: metadata?.backgroundColor ?? undefined,
                    }}
                ></span>
            </div>
        </div>
    );
}
