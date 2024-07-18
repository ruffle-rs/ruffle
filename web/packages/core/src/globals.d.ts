interface Error {
    avmStack?: string;
}
// Just updating this module seems to disable type checking for tsx-dom
// See https://github.com/Lusito/tsx-dom/issues/22#issuecomment-2236710966
// Because of that, all the other changes are unneeded, but they're what I think
// should be used if this worked properly
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
    xmlns?: string;
    fill?: string;
    stroke?: string;
    d?: string;
}
interface SVGTextElement {
    xmlns?: string;
    fill?: string;
    stroke?: string;
}
interface SVGCircleElement {
    xmlns?: string;
    fill?: string;
}
interface SVGDefsElement {
    xmlns?: string;
}
interface SVGGElement {
    xmlns?: string;
}
interface SVGLinearGradientElement {
    xmlns?: string;
}
interface SVGStopElement {
    xmlns?: string;
}
interface SVGUseElement {
    xmlns?: string;
}
interface HTMLInputElement {
    autocapitalize?: string;
    autocorrect?: string;
}
