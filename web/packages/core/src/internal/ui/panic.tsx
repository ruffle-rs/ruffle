import { text } from "../../i18n";
import { createRef } from "tsx-dom";

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
