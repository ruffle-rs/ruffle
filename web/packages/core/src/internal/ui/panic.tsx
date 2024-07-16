import { text, textAsParagraphs } from "../i18n";
import { createRef } from "tsx-dom";
import { buildInfo } from "../../build-info";
import { RUFFLE_ORIGIN } from "../constants";
import {
    InvalidOptionsError,
    InvalidSwfError,
    LoadRuffleWasmError,
    LoadSwfError,
} from "../errors";

interface PanicLink {
    type: "open_link";
    label: string;
    url: string;
}

interface PanicDetails {
    type: "show_details";
}

interface PanicCreateReport {
    type: "create_report";
}

type PanicAction = PanicLink | PanicDetails | PanicCreateReport;

function createPanicAction({
    action,
    showDetails,
    errorArray,
    errorText,
    swfUrl,
}: {
    action: PanicAction;
    showDetails: () => void;
    swfUrl: URL | undefined | null;
    errorArray: ErrorArray;
    errorText: string;
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
    } else if (action.type == "open_link") {
        return (
            <li>
                <a href={action.url} target="_top">
                    {action.label}
                </a>
            </li>
        );
    } else {
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
        return (
            <li>
                <a href={issueLink} target="_top">
                    {text("report-bug")}
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

export const CommonActions = {
    OpenDemo: {
        type: "open_link",
        url: RUFFLE_ORIGIN + "/demo",
        label: text("ruffle-demo"),
    } as PanicAction,

    DownloadDesktop: {
        type: "open_link",
        url: RUFFLE_ORIGIN + "/downloads#desktop-app",
        label: text("ruffle-desktop"),
    } as PanicAction,

    UpdateRuffle: {
        type: "open_link",
        url: RUFFLE_ORIGIN + "/downloads",
        label: text("update-ruffle"),
    } as PanicAction,

    CreateReport: {
        type: "create_report",
    } as PanicAction,

    ShowDetails: {
        type: "show_details",
    } as PanicAction,

    createReportOrUpdate(): PanicAction {
        return isBuildOutdated() ? this.UpdateRuffle : this.CreateReport;
    },

    openWiki(page: string, label?: string): PanicAction {
        return {
            type: "open_link",
            url: `https://github.com/ruffle-rs/ruffle/wiki/${page}`,
            label: label ?? text("ruffle-wiki"),
        };
    },
};

function createPanicError(error: Error | null): {
    body: HTMLDivElement;
    actions: PanicAction[];
} {
    if (error instanceof LoadSwfError) {
        if (error.swfUrl && !error.swfUrl.protocol.includes("http")) {
            // Loading a swf on the `file:` protocol
            return {
                body: textAsParagraphs("error-file-protocol"),
                actions: [
                    CommonActions.OpenDemo,
                    CommonActions.DownloadDesktop,
                ],
            };
        }

        if (
            window.location.origin === error.swfUrl?.origin ||
            // The extension's internal player page is not restricted by CORS
            window.location.protocol.includes("extension")
        ) {
            return {
                body: textAsParagraphs("error-swf-fetch"),
                actions: [CommonActions.ShowDetails],
            };
        }

        // This is a selfhosted build of Ruffle that tried to make a cross-origin request
        return {
            body: textAsParagraphs("error-swf-cors"),
            actions: [
                CommonActions.openWiki("Using-Ruffle#configure-cors-header"),
                CommonActions.ShowDetails,
            ],
        };
    }

    if (error instanceof InvalidSwfError) {
        return {
            body: textAsParagraphs("error-invalid-swf"),
            actions: [CommonActions.ShowDetails],
        };
    }

    if (error instanceof LoadRuffleWasmError) {
        if (window.location.protocol === "file:") {
            // Loading the wasm from the `file:` protocol
            return {
                body: textAsParagraphs("error-file-protocol"),
                actions: [
                    CommonActions.OpenDemo,
                    CommonActions.DownloadDesktop,
                ],
            };
        }

        const message = String(error.cause.message).toLowerCase();
        if (message.includes("mime")) {
            // Self hosted: Cannot load `.wasm` file - incorrect MIME type
            return {
                body: textAsParagraphs("error-wasm-mime-type"),
                actions: [
                    CommonActions.openWiki(
                        "Using-Ruffle#configure-webassembly-mime-type",
                    ),
                    CommonActions.ShowDetails,
                ],
            };
        }

        if (
            message.includes("networkerror") ||
            message.includes("failed to fetch")
        ) {
            // Self hosted: Cannot load `.wasm` file - CORS issues
            return {
                body: textAsParagraphs("error-wasm-cors"),
                actions: [
                    CommonActions.openWiki(
                        "Using-Ruffle#configure-cors-header",
                    ),
                    CommonActions.ShowDetails,
                ],
            };
        }

        if (message.includes("disallowed by embedder")) {
            // General error: Cannot load `.wasm` file - a native object / function is overridden
            return {
                body: textAsParagraphs("error-csp-conflict"),
                actions: [
                    CommonActions.openWiki("Using-Ruffle#configure-wasm-csp"),
                    CommonActions.ShowDetails,
                ],
            };
        }

        if (error.cause.name === "CompileError") {
            // Self hosted: Cannot load `.wasm` file - incorrect configuration or missing files
            return {
                body: textAsParagraphs("error-wasm-invalid"),
                actions: [
                    CommonActions.openWiki(
                        "Using-Ruffle#addressing-a-compileerror",
                    ),
                    CommonActions.ShowDetails,
                ],
            };
        }

        if (
            message.includes("could not download wasm module") &&
            error.cause.name === "TypeError"
        ) {
            // Usually a transient network error or botched deployment
            return {
                body: textAsParagraphs("error-wasm-download"),
                actions: [CommonActions.ShowDetails],
            };
        }

        if (error.cause.name === "TypeError") {
            // Self hosted: Cannot load `.wasm` file - a native object / function is overridden
            const body = textAsParagraphs("error-javascript-conflict");
            if (isBuildOutdated()) {
                body.appendChild(
                    textAsParagraphs("error-javascript-conflict-outdated", {
                        buildDate: buildInfo.buildDate,
                    }),
                );
            }
            return {
                body,
                actions: [
                    CommonActions.createReportOrUpdate(),
                    CommonActions.ShowDetails,
                ],
            };
        }

        if (
            navigator.userAgent.includes("Edg") &&
            message.includes("webassembly is not defined")
        ) {
            // Self hosted: User has disabled WebAssembly in Microsoft Edge through the
            // "Enhance your Security on the web" setting.
            return {
                body: textAsParagraphs("error-wasm-disabled-on-edge"),
                actions: [
                    CommonActions.openWiki(
                        "Frequently-Asked-Questions-For-Users#edge-webassembly-error",
                        text("more-info"),
                    ),
                    CommonActions.ShowDetails,
                ],
            };
        }

        // Self hosted: Cannot load `.wasm` file - file not found
        return {
            body: textAsParagraphs("error-wasm-not-found"),
            actions: [
                CommonActions.openWiki("Using-Ruffle#configuration-options"),
                CommonActions.ShowDetails,
            ],
        };
    }

    if (error instanceof InvalidOptionsError) {
        // General error: Incorrect JavaScript configuration
        return {
            body: textAsParagraphs("error-javascript-config"),
            actions: [
                CommonActions.openWiki("Using-Ruffle#javascript-api"),
                CommonActions.ShowDetails,
            ],
        };
    }

    return {
        body: textAsParagraphs("error-unknown", {
            buildDate: buildInfo.buildDate,
            outdated: String(isBuildOutdated),
        }),
        actions: [
            CommonActions.createReportOrUpdate(),
            CommonActions.ShowDetails,
        ],
    };
}

export function showPanicScreen(
    container: HTMLElement,
    error: Error | null,
    errorArray: ErrorArray,
    swfUrl: URL | undefined,
) {
    const errorText = errorArray.join("");
    let { body, actions } = createPanicError(error);

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
                            errorText,
                            errorArray,
                            swfUrl,
                        }),
                    )}
                </ul>
            </div>
        </div>,
    );
}
