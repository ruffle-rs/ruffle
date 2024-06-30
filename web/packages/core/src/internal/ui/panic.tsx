import { text } from "../../i18n";

export class PanicLinkInfo {
    constructor(
        public url: string = "#",
        public label: string = text("view-error-details"),
    ) {}
}

export function createErrorFooter(
    footerInfo: Array<PanicLinkInfo>,
): HTMLUListElement {
    const errorFooter = document.createElement("ul");
    for (const linkInfo of footerInfo) {
        const footerItem = document.createElement("li");
        const footerLink = document.createElement("a");
        footerLink.href = linkInfo.url;
        footerLink.textContent = linkInfo.label;
        if (linkInfo.url === "#") {
            footerLink.id = "panic-view-details";
        } else {
            footerLink.target = "_top";
        }
        footerItem.appendChild(footerLink);
        errorFooter.appendChild(footerItem);
    }
    return errorFooter;
}
