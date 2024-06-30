import { text } from "../../i18n";

export interface PanicLink {
    type: "open_link";
    label: string;
    url: string;
}

export interface PanicDetails {
    type: "show_details";
}

export type PanicAction = PanicLink | PanicDetails;

function createPanicAction(action: PanicAction) {
    if (action.type == "show_details") {
        return (
            <li>
                <a href="#" id="panic-view-details">
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

export function createErrorFooter(
    footerInfo: Array<PanicAction>,
): HTMLUListElement {
    return (<ul>{footerInfo.map(createPanicAction)}</ul>) as HTMLUListElement;
}
