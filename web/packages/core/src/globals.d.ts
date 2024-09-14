import "tsx-dom-types";

declare module "tsx-dom-types" {
    interface HTMLAttributes {
        // Removable after a release with https://github.com/Lusito/tsx-dom/pull/24
        autocapitalize?: string;
        // Type definition will not be updated in tsx-dom unless https://github.com/whatwg/html/pull/5841 is merged
        autocorrect?: string;
    }
    // Removable after a release with https://github.com/Lusito/tsx-dom/pull/23
    interface SVGAttributes {
        d?: string;
    }
}

declare global {
    interface Error {
        avmStack?: string;
    }
    // Per https://github.com/Lusito/tsx-dom/issues/22, attributes solely defined on SVGAttributes need type updates
    interface SVGElement {
        // Only SVGSVGElement would need to use xmlns if tsx-dom would use createElementNS without that
        xmlns?: string;
        fill?: string;
        stroke?: string;
    }
    interface SVGPathElement {
        d?: string;
    }
    interface HTMLInputElement {
        // Removable after a release with https://github.com/Lusito/tsx-dom/pull/24
        autocapitalize?: string;
        // Type definition will not be updated in tsx-dom unless https://github.com/whatwg/html/pull/5841 is merged
        autocorrect?: string;
    }
}
