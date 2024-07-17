interface Error {
    avmStack?: string;
}
module "tsx-dom-types" {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    interface HTMLAttributes {
        autocapitalize?: string;
        autocorrect?: string;
    }
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    interface SVGAttributes {
        d?: string;
    }
}
interface SVGSVGElement {
    xmlns?: string;
    scale?: string | number;
}
interface SVGPathElement {
    fill?: string;
    stroke?: string;
    d?: string;
}
interface SVGTextElement {
    fill?: string;
    stroke?: string;
}
interface SVGCircleElement {
    fill?: string;
}
interface HTMLInputElement {
    autocapitalize?: string;
    autocorrect?: string;
}
