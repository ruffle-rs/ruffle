import { text, textAsParagraphs } from "../../i18n";
import { createRef } from "tsx-dom";
import { buildInfo } from "../../build-info";
import { RUFFLE_ORIGIN } from "../constants";

export enum PanicError {
    Unknown,
    CSPConflict,
    FileProtocol,
    InvalidWasm,
    JavascriptConfiguration,
    JavascriptConflict,
    WasmCors,
    WasmDownload,
    WasmMimeType,
    WasmNotFound,
    WasmDisabledMicrosoftEdge,
    InvalidSwf,
    SwfFetchError,
    SwfCors,
}

interface PanicLink {
    type: "open_link";
    label: string;
    url: string;
}

interface PanicDetails {
    type: "show_details";
}

type PanicAction = PanicLink | PanicDetails;

function createPanicAction({
    action,
    showDetails,
}: {
    action: PanicAction;
    showDetails: () => void;
}) {
    if (action.type == "show_details") {
        const onClick = () => {
            showDetails();
            return false;
        };
        return (
            <li>
                <a href="#" id="panic-view-details" onClick={onClick}>
                    {text("view-error-details")}
                </a>
            </li>
        );
    } else {
        return (
            <li>
                <a href={action.url} target="_top">
                    {action.label}
                </a>
            </li>
        );
    }
}

function isBuildOutdated(): boolean {
    const buildDate = new Date(buildInfo.buildDate);
    const monthsPrior = new Date();
    monthsPrior.setMonth(monthsPrior.getMonth() - 6); // 6 months prior
    return monthsPrior > buildDate;
}

type ErrorArray = Array<string | null> & {
    stackIndex: number;
    avmStackIndex: number;
};

function createPanicError(
    errorIndex: number,
    errorText: string,
    errorArray: ErrorArray,
    swfUrl: URL | undefined,
) {
    let body: HTMLDivElement, actions: PanicAction[];
    switch (errorIndex) {
        case PanicError.FileProtocol:
            // General error: Running on the `file:` protocol
            body = textAsParagraphs("error-file-protocol");
            actions = [
                {
                    type: "open_link",
                    url: RUFFLE_ORIGIN + "/demo",
                    label: text("ruffle-demo"),
                },
                {
                    type: "open_link",
                    url: RUFFLE_ORIGIN + "/downloads#desktop-app",
                    label: text("ruffle-desktop"),
                },
            ];
            break;
        case PanicError.JavascriptConfiguration:
            // General error: Incorrect JavaScript configuration
            body = textAsParagraphs("error-javascript-config");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#javascript-api",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.WasmNotFound:
            // Self hosted: Cannot load `.wasm` file - file not found
            body = textAsParagraphs("error-wasm-not-found");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configuration-options",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.WasmMimeType:
            // Self hosted: Cannot load `.wasm` file - incorrect MIME type
            body = textAsParagraphs("error-wasm-mime-type");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-webassembly-mime-type",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.InvalidSwf:
            body = textAsParagraphs("error-invalid-swf");
            actions = [{ type: "show_details" }];
            break;
        case PanicError.SwfFetchError:
            body = textAsParagraphs("error-swf-fetch");
            actions = [{ type: "show_details" }];
            break;
        case PanicError.SwfCors:
            // Self hosted: Cannot load SWF file - CORS issues
            body = textAsParagraphs("error-swf-cors");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-cors-header",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.WasmCors:
            // Self hosted: Cannot load `.wasm` file - CORS issues
            body = textAsParagraphs("error-wasm-cors");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-cors-header",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.InvalidWasm:
            // Self hosted: Cannot load `.wasm` file - incorrect configuration or missing files
            body = textAsParagraphs("error-wasm-invalid");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#addressing-a-compileerror",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.WasmDownload:
            // Usually a transient network error or botched deployment
            body = textAsParagraphs("error-wasm-download");
            actions = [{ type: "show_details" }];
            break;
        case PanicError.WasmDisabledMicrosoftEdge:
            // Self hosted: User has disabled WebAssembly in Microsoft Edge through the
            // "Enhance your Security on the web" setting.
            body = textAsParagraphs("error-wasm-disabled-on-edge");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Frequently-Asked-Questions-For-Users#edge-webassembly-error",
                    label: text("more-info"),
                },
                { type: "show_details" },
            ];
            break;
        case PanicError.JavascriptConflict:
            // Self hosted: Cannot load `.wasm` file - a native object / function is overridden
            body = textAsParagraphs("error-javascript-conflict");
            if (isBuildOutdated()) {
                body.appendChild(
                    textAsParagraphs("error-javascript-conflict-outdated", {
                        buildDate: buildInfo.buildDate,
                    }),
                );
            }
            actions = [
                createReportAction({
                    errorText,
                    errorArray,
                    swfUrl,
                }),
                { type: "show_details" },
            ];
            break;
        case PanicError.CSPConflict:
            // General error: Cannot load `.wasm` file - a native object / function is overridden
            body = textAsParagraphs("error-csp-conflict");
            actions = [
                {
                    type: "open_link",
                    url: "https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-wasm-csp",
                    label: text("ruffle-wiki"),
                },
                { type: "show_details" },
            ];
            break;
        default:
            // Unknown error
            body = textAsParagraphs("error-unknown", {
                buildDate: buildInfo.buildDate,
                outdated: String(isBuildOutdated),
            });
            actions = [
                createReportAction({
                    errorText,
                    errorArray,
                    swfUrl,
                }),
                { type: "show_details" },
            ];
            break;
    }
    return { body, actions };
}

function createReportAction({
    swfUrl,
    errorText,
    errorArray,
}: {
    swfUrl: URL | undefined | null;
    errorArray: ErrorArray;
    errorText: string;
}): PanicAction {
    if (isBuildOutdated()) {
        return {
            type: "open_link",
            url: RUFFLE_ORIGIN + "/downloads#desktop-app",
            label: text("update-ruffle"),
        };
    }

    let url;
    if (document.location.protocol.includes("extension") && swfUrl) {
        url = swfUrl.href;
    } else {
        url = document.location.href;
    }

    // Remove query params for the issue title.
    url = url.split(/[?#]/, 1)[0]!;

    const issueTitle = `Error on ${url}`;
    let issueLink = `https://github.com/ruffle-rs/ruffle/issues/new?title=${encodeURIComponent(
        issueTitle,
    )}&template=error_report.md&labels=error-report&body=`;
    let issueBody = encodeURIComponent(errorText);
    if (
        errorArray.stackIndex > -1 &&
        String(issueLink + issueBody).length > 8195
    ) {
        // Strip the stack error from the array when the produced URL is way too long.
        // This should prevent "414 Request-URI Too Large" errors on GitHub.
        errorArray[errorArray.stackIndex] = null;
        if (errorArray.avmStackIndex > -1) {
            errorArray[errorArray.avmStackIndex] = null;
        }
        issueBody = encodeURIComponent(errorArray.join(""));
    }
    issueLink += issueBody;
    return {
        type: "open_link",
        url: issueLink,
        label: text("report-bug"),
    };
}

export function showPanicScreen(
    container: HTMLElement,
    errorIndex: number,
    errorArray: ErrorArray,
    swfUrl: URL | undefined,
) {
    const errorText = errorArray.join("");
    let { body, actions } = createPanicError(
        errorIndex,
        errorText,
        errorArray,
        swfUrl,
    );

    const panicBody = createRef<HTMLDivElement>();
    const showDetails = () => {
        panicBody.current!.classList.add("details");
        panicBody.current!.replaceChildren(
            <textarea readOnly>{errorText}</textarea>,
        );
    };

    container.textContent = "";
    container.appendChild(
        <div id="panic">
            <div id="panic-title">{text("panic-title")}</div>
            <div id="panic-body" ref={panicBody}>
                {body}
            </div>
            <div id="panic-footer">
                <ul>
                    {actions.map((action) =>
                        createPanicAction({
                            action,
                            showDetails,
                        }),
                    )}
                </ul>
            </div>
        </div>,
    );
}
