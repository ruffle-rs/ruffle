import { text } from "../../i18n";
import { createRef } from "tsx-dom";
import { buildInfo } from "../../build-info";
import { RUFFLE_ORIGIN } from "../constants";

export interface PanicLink {
    type: "open_link";
    label: string;
    url: string;
}

export interface PanicDetails {
    type: "show_details";
}

export type PanicAction = PanicLink | PanicDetails;

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

export function isBuildOutdated(): boolean {
    const buildDate = new Date(buildInfo.buildDate);
    const monthsPrior = new Date();
    monthsPrior.setMonth(monthsPrior.getMonth() - 6); // 6 months prior
    return monthsPrior > buildDate;
}

export type ErrorArray = Array<string | null> & {
    stackIndex: number;
    avmStackIndex: number;
};

export function createReportAction({
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
    errorBody: HTMLDivElement,
    actions: PanicAction[],
    errorText: string,
) {
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
                {errorBody}
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
